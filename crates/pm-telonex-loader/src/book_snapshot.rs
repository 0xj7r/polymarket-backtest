use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use arrow::array::{Array, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::SchemaRef;
use futures::TryStreamExt;
use object_store::{ObjectStore, path::Path as ObjectPath};
use parquet::arrow::ParquetRecordBatchStreamBuilder;
use parquet::arrow::async_reader::ParquetObjectReader;
use pm_types::{BookLevel, MarketId, ReplayEvent, ReplayFlags, tape::TAPE_DEPTH};

#[derive(Debug, Default, Clone)]
pub struct LoadStats {
    pub rows_total: usize,
    pub rows_emitted: usize,
    pub rows_null_top: usize,
    pub batches: usize,
    pub first_ts_ns: Option<i64>,
    pub last_ts_ns: Option<i64>,
}

/// Stream-load a Telonex book_snapshot parquet from S3 (or any object_store
/// backend) directly into `ReplayEvent`s. Top-5 levels are kept regardless of
/// whether the source file is `_5`, `_25`, or `_full` flat-format.
pub async fn load_book_snapshot_async(
    store: Arc<dyn ObjectStore>,
    path: ObjectPath,
    market_id: MarketId,
) -> Result<(Vec<ReplayEvent>, LoadStats)> {
    let reader = ParquetObjectReader::new(store, path.clone());

    let stream_builder = ParquetRecordBatchStreamBuilder::new(reader)
        .await
        .with_context(|| format!("open parquet stream {path}"))?;

    let schema: SchemaRef = stream_builder.schema().clone();
    let cols = TelonexColumnIndex::resolve(&schema)?;

    let mut stream = stream_builder.build().context("build record-batch stream")?;
    let mut stats = LoadStats::default();
    let mut out: Vec<ReplayEvent> = Vec::new();

    while let Some(batch) = stream
        .try_next()
        .await
        .context("read next record batch")?
    {
        process_batch(&batch, &cols, market_id, &mut out, &mut stats)?;
    }

    Ok((out, stats))
}

fn process_batch(
    batch: &RecordBatch,
    cols: &TelonexColumnIndex,
    market_id: MarketId,
    out: &mut Vec<ReplayEvent>,
    stats: &mut LoadStats,
) -> Result<()> {
    stats.batches += 1;
    let n = batch.num_rows();
    stats.rows_total += n;

    let ts = downcast_i64(batch, cols.timestamp_us, "timestamp_us")?;
    let bid_p = [
        downcast_str(batch, cols.bid_price[0], "bid_price_0")?,
        downcast_str(batch, cols.bid_price[1], "bid_price_1")?,
        downcast_str(batch, cols.bid_price[2], "bid_price_2")?,
        downcast_str(batch, cols.bid_price[3], "bid_price_3")?,
        downcast_str(batch, cols.bid_price[4], "bid_price_4")?,
    ];
    let bid_s = [
        downcast_str(batch, cols.bid_size[0], "bid_size_0")?,
        downcast_str(batch, cols.bid_size[1], "bid_size_1")?,
        downcast_str(batch, cols.bid_size[2], "bid_size_2")?,
        downcast_str(batch, cols.bid_size[3], "bid_size_3")?,
        downcast_str(batch, cols.bid_size[4], "bid_size_4")?,
    ];
    let ask_p = [
        downcast_str(batch, cols.ask_price[0], "ask_price_0")?,
        downcast_str(batch, cols.ask_price[1], "ask_price_1")?,
        downcast_str(batch, cols.ask_price[2], "ask_price_2")?,
        downcast_str(batch, cols.ask_price[3], "ask_price_3")?,
        downcast_str(batch, cols.ask_price[4], "ask_price_4")?,
    ];
    let ask_s = [
        downcast_str(batch, cols.ask_size[0], "ask_size_0")?,
        downcast_str(batch, cols.ask_size[1], "ask_size_1")?,
        downcast_str(batch, cols.ask_size[2], "ask_size_2")?,
        downcast_str(batch, cols.ask_size[3], "ask_size_3")?,
        downcast_str(batch, cols.ask_size[4], "ask_size_4")?,
    ];

    for i in 0..n {
        let bid0_valid = bid_p[0].is_valid(i);
        let ask0_valid = ask_p[0].is_valid(i);
        if !bid0_valid && !ask0_valid {
            stats.rows_null_top += 1;
            continue;
        }

        let mut bids = [BookLevel::default(); TAPE_DEPTH];
        let mut asks = [BookLevel::default(); TAPE_DEPTH];
        for lvl in 0..TAPE_DEPTH {
            bids[lvl] = read_level(bid_p[lvl], bid_s[lvl], i);
            asks[lvl] = read_level(ask_p[lvl], ask_s[lvl], i);
        }

        let yes_bid = bids[0].price;
        let yes_ask = asks[0].price;
        let yes_mid = if yes_bid > 0.0 && yes_ask > 0.0 {
            0.5 * (yes_bid + yes_ask)
        } else {
            yes_bid.max(yes_ask)
        };

        let ts_us = ts.value(i);
        let ts_ns = ts_us.saturating_mul(1_000);

        stats.first_ts_ns.get_or_insert(ts_ns);
        stats.last_ts_ns = Some(ts_ns);

        out.push(ReplayEvent {
            ts_ns,
            market_id,
            yes_mid,
            yes_bid,
            yes_ask,
            volume: 0.0,
            bids,
            asks,
            spot_price: 0.0,
            flags: ReplayFlags::BOOK_UPDATE,
        });
        stats.rows_emitted += 1;
    }
    Ok(())
}

#[derive(Debug)]
struct TelonexColumnIndex {
    timestamp_us: usize,
    bid_price: [usize; TAPE_DEPTH],
    bid_size: [usize; TAPE_DEPTH],
    ask_price: [usize; TAPE_DEPTH],
    ask_size: [usize; TAPE_DEPTH],
}

impl TelonexColumnIndex {
    fn resolve(schema: &SchemaRef) -> Result<Self> {
        let find = |name: &str| -> Result<usize> {
            schema
                .fields()
                .iter()
                .position(|f| f.name() == name)
                .ok_or_else(|| anyhow!("missing column: {name}"))
        };
        let timestamp_us = find("timestamp_us")?;
        let mut bid_price = [0usize; TAPE_DEPTH];
        let mut bid_size = [0usize; TAPE_DEPTH];
        let mut ask_price = [0usize; TAPE_DEPTH];
        let mut ask_size = [0usize; TAPE_DEPTH];
        for i in 0..TAPE_DEPTH {
            bid_price[i] = find(&format!("bid_price_{i}"))?;
            bid_size[i] = find(&format!("bid_size_{i}"))?;
            ask_price[i] = find(&format!("ask_price_{i}"))?;
            ask_size[i] = find(&format!("ask_size_{i}"))?;
        }
        Ok(Self {
            timestamp_us,
            bid_price,
            bid_size,
            ask_price,
            ask_size,
        })
    }
}

fn downcast_i64<'a>(batch: &'a RecordBatch, idx: usize, name: &str) -> Result<&'a Int64Array> {
    batch
        .column(idx)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| anyhow!("column {name} not int64"))
}

fn downcast_str<'a>(batch: &'a RecordBatch, idx: usize, name: &str) -> Result<&'a StringArray> {
    let col = batch.column(idx);
    col.as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("column {name} not utf8 (got {:?})", col.data_type()))
}

fn read_level(prices: &StringArray, sizes: &StringArray, row: usize) -> BookLevel {
    let price = if prices.is_valid(row) {
        prices.value(row).parse::<f32>().unwrap_or(0.0)
    } else {
        0.0
    };
    let size = if sizes.is_valid(row) {
        sizes.value(row).parse::<f32>().unwrap_or(0.0)
    } else {
        0.0
    };
    BookLevel { price, size }
}
