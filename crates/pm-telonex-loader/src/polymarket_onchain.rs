//! Polymarket onchain_fills loader.
//!
//! Each row is a settled trade on Polygon. Useful columns:
//!   - block_timestamp_us: settlement time (int64 µs since epoch)
//!   - maker, taker: wallet addresses (lower-cased hex with `0x` prefix)
//!   - maker_side, taker_side: "buy" / "sell" relative to maker/taker asset_id
//!   - amount, price: decimal strings
//!
//! Use case: filter to a tracked whale wallet to derive a "whale-flow"
//! feature aligned to the polymarket book tape.
//!
//! NOTE: Telonex's onchain_fills archive only covers ~Feb 12 → Apr 28 2026.
//! Loader works on any date that has data; callers should fall back to an
//! empty `WhaleFlowHistory` when no file exists.

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use arrow::array::{Array, BooleanArray, Int64Array, RecordBatch, StringArray};
use arrow::datatypes::SchemaRef;
use futures::TryStreamExt;
use object_store::{ObjectStore, path::Path as ObjectPath};
use parquet::arrow::ParquetRecordBatchStreamBuilder;
use parquet::arrow::async_reader::ParquetObjectReader;
use serde::{Deserialize, Serialize};

use crate::s3::TelonexStore;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnchainFill {
    pub ts_ns: i64,
    pub maker: String,
    pub taker: String,
    /// Maker side relative to maker_asset_id ("buy"/"sell").
    pub maker_side: String,
    /// Taker side relative to taker_asset_id.
    pub taker_side: String,
    pub price: f32,
    pub size: f32,
    pub mirrored: bool,
}

#[derive(Debug, Default, Clone)]
pub struct OnchainLoadStats {
    pub rows_total: usize,
    pub rows_emitted: usize,
    pub batches: usize,
    pub first_ts_ns: Option<i64>,
    pub last_ts_ns: Option<i64>,
}

pub fn build_pm_onchain_prefix(date: &str, asset_id: &str) -> ObjectPath {
    ObjectPath::from(format!(
        "raw/telonex/exchange=polymarket/channel=onchain_fills/date={date}/asset_id={asset_id}/"
    ))
}

pub async fn resolve_pm_onchain_day(
    store: &TelonexStore,
    date: &str,
    asset_id: &str,
) -> Result<ObjectPath> {
    let prefix = build_pm_onchain_prefix(date, asset_id);
    let files = store.list_prefix(&prefix, 8).await?;
    files
        .into_iter()
        .find(|p| p.as_ref().ends_with(".parquet"))
        .ok_or_else(|| anyhow!("no .parquet under {prefix}"))
}

/// Stream Polymarket onchain_fills parquet into `OnchainFill` records.
/// If `wallet_filter` is `Some`, retain only fills where maker or taker
/// matches (lowercased compare).
pub async fn load_pm_onchain_async(
    store: Arc<dyn ObjectStore>,
    path: ObjectPath,
    wallet_filter: Option<&str>,
) -> Result<(Vec<OnchainFill>, OnchainLoadStats)> {
    let reader = ParquetObjectReader::new(store, path.clone());
    let stream_builder = ParquetRecordBatchStreamBuilder::new(reader)
        .await
        .with_context(|| format!("open parquet stream {path}"))?;
    let schema: SchemaRef = stream_builder.schema().clone();
    let cols = OnchainColumns::resolve(&schema)?;
    let mut stream = stream_builder.build().context("build batch stream")?;
    let mut out: Vec<OnchainFill> = Vec::new();
    let mut stats = OnchainLoadStats::default();
    let wallet_lower = wallet_filter.map(|s| s.to_ascii_lowercase());
    while let Some(batch) = stream.try_next().await.context("read next record batch")? {
        process(&batch, &cols, wallet_lower.as_deref(), &mut out, &mut stats)?;
    }
    Ok((out, stats))
}

#[derive(Debug)]
struct OnchainColumns {
    ts_us: usize,
    maker: usize,
    taker: usize,
    maker_side: usize,
    taker_side: usize,
    amount: usize,
    price: usize,
    mirrored: usize,
}

impl OnchainColumns {
    fn resolve(schema: &SchemaRef) -> Result<Self> {
        let find = |name: &str| -> Result<usize> {
            schema
                .fields()
                .iter()
                .position(|f| f.name() == name)
                .ok_or_else(|| anyhow!("missing column: {name}"))
        };
        Ok(Self {
            ts_us: find("block_timestamp_us")?,
            maker: find("maker")?,
            taker: find("taker")?,
            maker_side: find("maker_side")?,
            taker_side: find("taker_side")?,
            amount: find("amount")?,
            price: find("price")?,
            mirrored: find("mirrored")?,
        })
    }
}

fn process(
    batch: &RecordBatch,
    cols: &OnchainColumns,
    wallet_filter: Option<&str>,
    out: &mut Vec<OnchainFill>,
    stats: &mut OnchainLoadStats,
) -> Result<()> {
    stats.batches += 1;
    let n = batch.num_rows();
    stats.rows_total += n;

    let ts = batch
        .column(cols.ts_us)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| anyhow!("ts not int64"))?;
    let maker = batch
        .column(cols.maker)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("maker not utf8"))?;
    let taker = batch
        .column(cols.taker)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("taker not utf8"))?;
    let amount = batch
        .column(cols.amount)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("amount not utf8"))?;
    let price = batch
        .column(cols.price)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("price not utf8"))?;
    let mirrored = batch
        .column(cols.mirrored)
        .as_any()
        .downcast_ref::<BooleanArray>()
        .ok_or_else(|| anyhow!("mirrored not bool"))?;

    // maker_side / taker_side may be dictionary-encoded. We handle plain
    // StringArray here; dictionary support can be added if needed.
    let maker_side = batch
        .column(cols.maker_side)
        .as_any()
        .downcast_ref::<StringArray>();
    let taker_side = batch
        .column(cols.taker_side)
        .as_any()
        .downcast_ref::<StringArray>();

    for i in 0..n {
        if !ts.is_valid(i) || !maker.is_valid(i) || !taker.is_valid(i) {
            continue;
        }
        let mk = maker.value(i).to_ascii_lowercase();
        let tk = taker.value(i).to_ascii_lowercase();
        if let Some(f) = wallet_filter {
            if mk != f && tk != f {
                continue;
            }
        }
        let Ok(p) = price.value(i).parse::<f32>() else {
            continue;
        };
        let Ok(s) = amount.value(i).parse::<f32>() else {
            continue;
        };
        if !p.is_finite() || p <= 0.0 || !s.is_finite() || s <= 0.0 {
            continue;
        }
        let ts_ns = ts.value(i).saturating_mul(1_000);
        stats.first_ts_ns.get_or_insert(ts_ns);
        stats.last_ts_ns = Some(ts_ns);
        let mk_side = maker_side
            .and_then(|a| a.is_valid(i).then(|| a.value(i).to_string()))
            .unwrap_or_default();
        let tk_side = taker_side
            .and_then(|a| a.is_valid(i).then(|| a.value(i).to_string()))
            .unwrap_or_default();
        out.push(OnchainFill {
            ts_ns,
            maker: mk,
            taker: tk,
            maker_side: mk_side,
            taker_side: tk_side,
            price: p,
            size: s,
            mirrored: mirrored.value(i),
        });
        stats.rows_emitted += 1;
    }
    Ok(())
}

/// Compute net buy-minus-sell amount attributable to `wallet` in the trailing
/// `lookback_ns` window ending at `now_ns`. Positive = wallet net-bought.
pub fn whale_net_flow(fills: &[OnchainFill], wallet: &str, now_ns: i64, lookback_ns: i64) -> f64 {
    let start = now_ns - lookback_ns;
    let wallet_lower = wallet.to_ascii_lowercase();
    let mut net = 0.0f64;
    for f in fills {
        if f.ts_ns < start || f.ts_ns > now_ns {
            continue;
        }
        if f.mirrored {
            continue; // skip the mirror row to avoid double-counting
        }
        // Determine the wallet's direction. We treat "buy" as +amount,
        // "sell" as -amount.
        if f.maker == wallet_lower {
            let sign = if f.maker_side == "buy" { 1.0 } else { -1.0 };
            net += sign * f.size as f64;
        } else if f.taker == wallet_lower {
            let sign = if f.taker_side == "buy" { 1.0 } else { -1.0 };
            net += sign * f.size as f64;
        }
    }
    net
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill(
        ts_ns: i64,
        maker: &str,
        taker: &str,
        maker_side: &str,
        size: f32,
        mirrored: bool,
    ) -> OnchainFill {
        OnchainFill {
            ts_ns,
            maker: maker.to_string(),
            taker: taker.to_string(),
            maker_side: maker_side.to_string(),
            taker_side: if maker_side == "buy" {
                "sell".into()
            } else {
                "buy".into()
            },
            price: 0.5,
            size,
            mirrored,
        }
    }

    #[test]
    fn whale_flow_counts_maker_and_taker_correctly() {
        let whale = "0xabc";
        let fills = vec![
            fill(1_000_000_000, "0xabc", "0xdef", "buy", 10.0, false),
            fill(2_000_000_000, "0xdef", "0xabc", "buy", 5.0, false),
            fill(3_000_000_000, "0xabc", "0xdef", "sell", 3.0, false),
            // mirrored row — should be skipped
            fill(4_000_000_000, "0xabc", "0xdef", "buy", 100.0, true),
        ];
        let net = whale_net_flow(&fills, whale, 5_000_000_000, 10_000_000_000);
        // maker buy 10 (+10) + taker buy 5 (sell side; -5) + maker sell 3 (-3) = +2
        // Wait: as taker on row 2: taker_side is opposite of maker_side="buy", so taker_side="sell" → sign -1
        // Result: 10 (maker buy) + (-5) (taker sell) + (-3) (maker sell) = +2
        assert!((net - 2.0).abs() < 1e-9, "got {net}");
    }
}
