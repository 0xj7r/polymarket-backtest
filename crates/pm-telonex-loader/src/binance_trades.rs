//! Binance aggregated-trades parquet loader (S3 streaming).
//!
//! Schema (from raw/binance/exchange=binance/channel=agg_trades/symbol=*/date=*/...parquet):
//! ```text
//! exchange: string
//! symbol:   string
//! agg_trade_id: i64
//! price:    string (decimal)
//! quantity: string (decimal)
//! first_trade_id: i64
//! last_trade_id: i64
//! transact_time_ms: i64   (NOTE: actually microseconds despite the name)
//! is_buyer_maker: bool
//! is_best_match: bool
//! ```

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use arrow::array::{Array, BooleanArray, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::SchemaRef;
use futures::TryStreamExt;
use object_store::{ObjectStore, path::Path as ObjectPath};
use parquet::arrow::ParquetRecordBatchStreamBuilder;
use parquet::arrow::async_reader::ParquetObjectReader;
use pm_types::SpotTick;

use crate::s3::TelonexStore;

#[derive(Debug, Default, Clone)]
pub struct BinanceLoadStats {
    pub rows_total: usize,
    pub rows_emitted: usize,
    pub batches: usize,
    pub first_ts_ns: Option<i64>,
    pub last_ts_ns: Option<i64>,
}

pub fn build_binance_prefix(channel: &str, symbol: &str, date: &str) -> ObjectPath {
    ObjectPath::from(format!(
        "raw/binance/exchange=binance/channel={channel}/symbol={symbol}/date={date}/"
    ))
}

/// Resolve the parquet file for one (channel, symbol, date) Binance cell.
pub async fn resolve_binance_day(
    store: &TelonexStore,
    channel: &str,
    symbol: &str,
    date: &str,
) -> Result<ObjectPath> {
    let prefix = build_binance_prefix(channel, symbol, date);
    let files = store.list_prefix(&prefix, 8).await?;
    files
        .into_iter()
        .find(|p| p.as_ref().ends_with(".parquet"))
        .ok_or_else(|| anyhow!("no .parquet under {prefix}"))
}

/// Stream a Binance aggTrades parquet into `SpotTick`s.
pub async fn load_binance_agg_trades_async(
    store: Arc<dyn ObjectStore>,
    path: ObjectPath,
) -> Result<(Vec<SpotTick>, BinanceLoadStats)> {
    let reader = ParquetObjectReader::new(store, path.clone());
    let stream_builder = ParquetRecordBatchStreamBuilder::new(reader)
        .await
        .with_context(|| format!("open parquet stream {path}"))?;
    let schema: SchemaRef = stream_builder.schema().clone();
    let cols = BinanceColumns::resolve(&schema)?;
    let mut stream = stream_builder.build().context("build batch stream")?;
    let mut out: Vec<SpotTick> = Vec::new();
    let mut stats = BinanceLoadStats::default();

    while let Some(batch) = stream
        .try_next()
        .await
        .context("read next record batch")?
    {
        process(&batch, &cols, &mut out, &mut stats)?;
    }
    Ok((out, stats))
}

#[derive(Debug)]
struct BinanceColumns {
    transact_time: usize,
    price: usize,
    quantity: usize,
    is_buyer_maker: usize,
}

impl BinanceColumns {
    fn resolve(schema: &SchemaRef) -> Result<Self> {
        let find = |name: &str| -> Result<usize> {
            schema
                .fields()
                .iter()
                .position(|f| f.name() == name)
                .ok_or_else(|| anyhow!("missing column: {name}"))
        };
        Ok(Self {
            // Despite the column name, Telonex publishes Binance aggTrade
            // timestamps in microseconds, not milliseconds.
            transact_time: find("transact_time_ms")?,
            price: find("price")?,
            quantity: find("quantity")?,
            is_buyer_maker: find("is_buyer_maker")?,
        })
    }
}

fn process(
    batch: &RecordBatch,
    cols: &BinanceColumns,
    out: &mut Vec<SpotTick>,
    stats: &mut BinanceLoadStats,
) -> Result<()> {
    stats.batches += 1;
    let n = batch.num_rows();
    stats.rows_total += n;

    let ts = batch
        .column(cols.transact_time)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| anyhow!("transact_time_ms not int64"))?;
    let price = batch
        .column(cols.price)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("price not utf8"))?;
    let qty = batch
        .column(cols.quantity)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("quantity not utf8"))?;
    let bm = batch
        .column(cols.is_buyer_maker)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| anyhow!("is_buyer_maker not bool"))?;

    for i in 0..n {
        if !ts.is_valid(i) || !price.is_valid(i) {
            continue;
        }
        let Ok(p) = price.value(i).parse::<f64>() else { continue };
        if !p.is_finite() || p <= 0.0 {
            continue;
        }
        let q = if qty.is_valid(i) {
            qty.value(i).parse::<f32>().unwrap_or(0.0)
        } else {
            0.0
        };
        let ts_us = ts.value(i);
        let ts_ns = ts_us.saturating_mul(1_000);
        stats.first_ts_ns.get_or_insert(ts_ns);
        stats.last_ts_ns = Some(ts_ns);
        out.push(SpotTick {
            ts_ns,
            price: p,
            quantity: q,
            is_buyer_maker: bm.value(i),
        });
        stats.rows_emitted += 1;
    }
    Ok(())
}
