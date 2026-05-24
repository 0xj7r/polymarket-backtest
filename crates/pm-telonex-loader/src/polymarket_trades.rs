//! Polymarket trades-channel loader. Each row is a single taker fill with an
//! explicit aggressor `side` ("buy" = taker hit the ask, "sell" = taker hit
//! the bid). Trade-flow imbalance is far more predictive of imminent direction
//! than book-side imbalance because it captures actual flow, not just resting
//! depth.

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use arrow::array::{Array, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::SchemaRef;
use futures::TryStreamExt;
use object_store::{ObjectStore, path::Path as ObjectPath};
use parquet::arrow::ParquetRecordBatchStreamBuilder;
use parquet::arrow::async_reader::ParquetObjectReader;
use pm_types::TradeTick;

use crate::s3::TelonexStore;

#[derive(Debug, Default, Clone)]
pub struct TradesLoadStats {
    pub rows_total: usize,
    pub rows_emitted: usize,
    pub buy_count: usize,
    pub sell_count: usize,
    pub batches: usize,
    pub first_ts_ns: Option<i64>,
    pub last_ts_ns: Option<i64>,
}

pub fn build_pm_trades_prefix(date: &str, asset_id: &str) -> ObjectPath {
    ObjectPath::from(format!(
        "raw/telonex/exchange=polymarket/channel=trades/date={date}/asset_id={asset_id}/"
    ))
}

pub async fn resolve_pm_trades_day(
    store: &TelonexStore,
    date: &str,
    asset_id: &str,
) -> Result<ObjectPath> {
    let prefix = build_pm_trades_prefix(date, asset_id);
    let files = store.list_prefix(&prefix, 8).await?;
    files
        .into_iter()
        .find(|p| p.as_ref().ends_with(".parquet"))
        .ok_or_else(|| anyhow!("no .parquet under {prefix}"))
}

/// Stream a Polymarket trades parquet into `TradeTick`s.
pub async fn load_pm_trades_async(
    store: Arc<dyn ObjectStore>,
    path: ObjectPath,
) -> Result<(Vec<TradeTick>, TradesLoadStats)> {
    let reader = ParquetObjectReader::new(store, path.clone());
    let stream_builder = ParquetRecordBatchStreamBuilder::new(reader)
        .await
        .with_context(|| format!("open parquet stream {path}"))?;
    let schema: SchemaRef = stream_builder.schema().clone();
    let cols = TradesColumns::resolve(&schema)?;
    let mut stream = stream_builder.build().context("build batch stream")?;
    let mut out: Vec<TradeTick> = Vec::new();
    let mut stats = TradesLoadStats::default();
    while let Some(batch) = stream.try_next().await.context("read next record batch")? {
        process(&batch, &cols, &mut out, &mut stats)?;
    }
    Ok((out, stats))
}

#[derive(Debug)]
struct TradesColumns {
    timestamp_us: usize,
    price: usize,
    size: usize,
    side: usize,
}

impl TradesColumns {
    fn resolve(schema: &SchemaRef) -> Result<Self> {
        let find = |name: &str| -> Result<usize> {
            schema
                .fields()
                .iter()
                .position(|f| f.name() == name)
                .ok_or_else(|| anyhow!("missing column: {name}"))
        };
        Ok(Self {
            timestamp_us: find("timestamp_us")?,
            price: find("price")?,
            size: find("size")?,
            side: find("side")?,
        })
    }
}

fn process(
    batch: &RecordBatch,
    cols: &TradesColumns,
    out: &mut Vec<TradeTick>,
    stats: &mut TradesLoadStats,
) -> Result<()> {
    stats.batches += 1;
    let n = batch.num_rows();
    stats.rows_total += n;
    let ts = batch
        .column(cols.timestamp_us)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| anyhow!("timestamp_us not int64"))?;
    let price = batch
        .column(cols.price)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("price not utf8"))?;
    let size = batch
        .column(cols.size)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("size not utf8"))?;
    let side = batch
        .column(cols.side)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("side not utf8"))?;
    for i in 0..n {
        if !ts.is_valid(i) || !price.is_valid(i) || !size.is_valid(i) || !side.is_valid(i) {
            continue;
        }
        let Ok(p) = price.value(i).parse::<f32>() else {
            continue;
        };
        let Ok(s) = size.value(i).parse::<f32>() else {
            continue;
        };
        if !p.is_finite() || p <= 0.0 || !s.is_finite() || s <= 0.0 {
            continue;
        }
        let aggressor_buy = match side.value(i) {
            "buy" => true,
            "sell" => false,
            _ => continue,
        };
        let ts_ns = ts.value(i).saturating_mul(1_000);
        stats.first_ts_ns.get_or_insert(ts_ns);
        stats.last_ts_ns = Some(ts_ns);
        if aggressor_buy {
            stats.buy_count += 1;
        } else {
            stats.sell_count += 1;
        }
        out.push(TradeTick {
            ts_ns,
            price: p,
            size: s,
            aggressor_buy,
        });
        stats.rows_emitted += 1;
    }
    Ok(())
}
