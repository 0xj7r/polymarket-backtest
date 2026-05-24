//! prep-cache — download per-market parquets from S3 to a local mirror.
//!
//! The mirror layout matches the S3 prefix scheme exactly so any code that
//! reads via `object_store` (S3 or LocalFileSystem) works unchanged. Once
//! cached, walk-forward with `--local-cache-dir` reads mmap-style off disk
//! and a 9-day backtest runs in seconds instead of hours.

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use object_store::{GetOptions, GetResult, ObjectStore, path::Path as ObjectPath};
use pm_telonex_loader::TelonexStore;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::io::AsyncWriteExt;

use crate::discovery::MarketHandle;

#[derive(Debug, Clone)]
pub struct PrepCacheConfig {
    pub cache_dir: PathBuf,
    pub spot_symbol: String,
    pub max_concurrent: usize,
    /// If true, skip files already present locally.
    pub skip_existing: bool,
}

pub async fn run_prep_cache(
    store: &TelonexStore,
    markets: &[MarketHandle],
    cfg: &PrepCacheConfig,
) -> Result<()> {
    std::fs::create_dir_all(&cfg.cache_dir)
        .with_context(|| format!("mkdir -p {}", cfg.cache_dir.display()))?;

    // Build the set of (s3_key, local_path) pairs we need.
    let mut targets: Vec<(ObjectPath, PathBuf)> = Vec::new();

    // 1. Spot history per unique date.
    let unique_dates: BTreeSet<String> = markets.iter().map(|m| m.date.clone()).collect();
    if !cfg.spot_symbol.is_empty() {
        for date in &unique_dates {
            let prefix = ObjectPath::from(format!(
                "raw/binance/exchange=binance/channel=agg_trades/symbol={}/date={}/",
                cfg.spot_symbol, date
            ));
            // Resolve via list (one file per leaf prefix).
            match store.list_prefix(&prefix, 8).await {
                Ok(files) => {
                    if let Some(f) = files.into_iter().find(|p| p.as_ref().ends_with(".parquet")) {
                        let local = cfg.cache_dir.join(f.as_ref());
                        targets.push((f, local));
                    } else {
                        tracing::warn!(date, "no spot file");
                    }
                }
                Err(e) => tracing::warn!(date, error = %e, "spot list failed"),
            }
        }
    }

    // 2. Book + trades per market.
    for m in markets {
        for channel in ["book_snapshot_25", "trades"] {
            let prefix = ObjectPath::from(format!(
                "raw/telonex/exchange=polymarket/channel={}/date={}/asset_id={}/",
                channel, m.date, m.asset_id
            ));
            match store.list_prefix(&prefix, 4).await {
                Ok(files) => {
                    if let Some(f) = files.into_iter().find(|p| p.as_ref().ends_with(".parquet")) {
                        let local = cfg.cache_dir.join(f.as_ref());
                        targets.push((f, local));
                    }
                }
                Err(e) => tracing::debug!(market = %m.slug, channel, error = %e, "list failed"),
            }
        }
    }

    let total = targets.len();
    tracing::info!(total, cache_dir = ?cfg.cache_dir, "prep-cache targets resolved");
    let store_inner = store.store();
    let done = Arc::new(AtomicUsize::new(0));
    let skipped = Arc::new(AtomicUsize::new(0));
    let errored = Arc::new(AtomicUsize::new(0));
    let bytes_downloaded = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let skip_existing = cfg.skip_existing;

    let results = futures::stream::iter(targets.into_iter().map(|(key, local)| {
        let store_inner = store_inner.clone();
        let done = done.clone();
        let skipped = skipped.clone();
        let bytes_downloaded = bytes_downloaded.clone();
        async move {
            if skip_existing && local.exists() {
                skipped.fetch_add(1, Ordering::Relaxed);
                return Ok::<_, anyhow::Error>(());
            }
            if let Some(parent) = local.parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
            let resp: GetResult = store_inner
                .get_opts(&key, GetOptions::default())
                .await
                .with_context(|| format!("get {key}"))?;
            let bytes = resp
                .bytes()
                .await
                .with_context(|| format!("read bytes {key}"))?;
            bytes_downloaded.fetch_add(bytes.len() as u64, Ordering::Relaxed);
            let mut f = tokio::fs::File::create(&local)
                .await
                .with_context(|| format!("create {}", local.display()))?;
            f.write_all(&bytes)
                .await
                .with_context(|| format!("write {}", local.display()))?;
            f.flush().await.ok();
            let n = done.fetch_add(1, Ordering::Relaxed) + 1;
            if n % 100 == 0 {
                tracing::info!(
                    done = n,
                    total,
                    mb = bytes_downloaded.load(Ordering::Relaxed) as f64 / 1_048_576.0,
                    "prep-cache progress"
                );
            }
            Ok(())
        }
    }))
    .buffer_unordered(cfg.max_concurrent)
    .collect::<Vec<_>>()
    .await;

    for r in &results {
        if r.is_err() {
            errored.fetch_add(1, Ordering::Relaxed);
        }
    }

    let done_n = done.load(Ordering::Relaxed);
    let skipped_n = skipped.load(Ordering::Relaxed);
    let errored_n = errored.load(Ordering::Relaxed);
    let bytes_n = bytes_downloaded.load(Ordering::Relaxed);
    tracing::info!(
        done = done_n,
        skipped = skipped_n,
        errored = errored_n,
        total,
        mb_downloaded = bytes_n as f64 / 1_048_576.0,
        "prep-cache complete"
    );
    println!(
        "prep-cache: {done_n} downloaded + {skipped_n} skipped + {errored_n} errors / {total} total, {:.1} MB",
        bytes_n as f64 / 1_048_576.0
    );
    if errored_n > 0 {
        return Err(anyhow!("{errored_n} downloads failed"));
    }
    Ok(())
}
