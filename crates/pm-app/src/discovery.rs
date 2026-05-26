//! Market discovery: list available asset_ids under a date prefix in S3 and
//! resolve their slug + resolution timestamp via the Telonex availability API.
//!
//! Output is JSONL of `MarketHandle` rows that `walk-forward` reads.

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use object_store::path::Path as ObjectPath;
use pm_telonex_loader::TelonexStore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketHandle {
    pub asset_id: String,
    pub slug: String,
    /// Resolution timestamp (Unix seconds, UTC).
    pub close_ts: i64,
    /// Outcome label as returned by Telonex (e.g. "Up", "Down", "Yes", "No").
    pub outcome: String,
    /// Date partition (`YYYY-MM-DD`).
    pub date: String,
}

#[derive(Debug, Deserialize)]
struct AvailabilityResponse {
    #[allow(dead_code)]
    asset_id: String,
    slug: String,
    outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AvailabilityCacheEntry {
    asset_id: String,
    slug: String,
    outcome: String,
}

type AvailabilityCache = HashMap<String, (String, String)>;

/// List all `asset_id=...` sub-prefixes under
/// `raw/telonex/exchange=polymarket/channel=book_snapshot_25/date=DATE/`.
pub async fn list_asset_ids_for_day(store: &TelonexStore, date: &str) -> Result<Vec<String>> {
    let prefix = ObjectPath::from(format!(
        "raw/telonex/exchange=polymarket/channel=book_snapshot_25/date={date}/"
    ));
    let inner = store.store();
    let mut listing = inner
        .list_with_delimiter(Some(&prefix))
        .await
        .with_context(|| format!("list_with_delimiter {prefix}"))?;
    listing.common_prefixes.sort();
    let asset_ids = listing
        .common_prefixes
        .iter()
        .filter_map(|p| {
            let raw = p.as_ref();
            let last = raw.trim_end_matches('/').rsplit('/').next()?;
            last.strip_prefix("asset_id=").map(|s| s.to_string())
        })
        .collect();
    Ok(asset_ids)
}

/// List `asset_id=...` directories from a local cache mirror produced by
/// `prep-cache`.
pub fn list_asset_ids_for_local_cache_day(cache_dir: &Path, date: &str) -> Result<Vec<String>> {
    let dir = cache_dir
        .join("raw/telonex/exchange=polymarket/channel=book_snapshot_25")
        .join(format!("date={date}"));
    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("read local cache day directory {}", dir.display()))?;
    let mut asset_ids = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("read entry under {}", dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("read file type for {}", entry.path().display()))?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if let Some(asset_id) = name.strip_prefix("asset_id=") {
            asset_ids.push(asset_id.to_string());
        }
    }
    asset_ids.sort();
    Ok(asset_ids)
}

/// Hit the public `/v1/availability/polymarket?asset_id=X` endpoint; extract
/// slug + outcome. No auth required. Retries on 429 with exponential backoff.
pub async fn fetch_availability(
    client: &reqwest::Client,
    asset_id: &str,
) -> Result<(String, String)> {
    let url = format!("https://api.telonex.io/v1/availability/polymarket?asset_id={asset_id}");
    let mut backoff = std::time::Duration::from_secs(1);
    for attempt in 0..10 {
        let resp = client
            .get(&url)
            .send()
            .await
            .with_context(|| format!("availability GET {asset_id}"))?;
        if resp.status().as_u16() == 429 {
            if attempt == 9 {
                return Err(anyhow!("availability 429 (max retries) for {asset_id}"));
            }
            let retry_after = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .map(std::time::Duration::from_secs);
            tokio::time::sleep(retry_after.unwrap_or(backoff)).await;
            backoff = (backoff * 2).min(std::time::Duration::from_secs(60));
            continue;
        }
        if !resp.status().is_success() {
            return Err(anyhow!("availability {} for {}", resp.status(), asset_id));
        }
        let body: AvailabilityResponse = resp
            .json()
            .await
            .with_context(|| format!("decode availability for {asset_id}"))?;
        return Ok((body.slug, body.outcome));
    }
    Err(anyhow!("availability exhausted retries for {asset_id}"))
}

fn load_availability_cache(path: &Path) -> Result<AvailabilityCache> {
    match std::fs::File::open(path) {
        Ok(file) => {
            let mut out = HashMap::new();
            for (idx, line) in BufReader::new(file).lines().enumerate() {
                let line = line.with_context(|| {
                    format!(
                        "read availability cache line {} from {}",
                        idx + 1,
                        path.display()
                    )
                })?;
                if line.trim().is_empty() {
                    continue;
                }
                let row: AvailabilityCacheEntry =
                    serde_json::from_str(&line).with_context(|| {
                        format!(
                            "decode availability cache line {} from {}",
                            idx + 1,
                            path.display()
                        )
                    })?;
                out.insert(row.asset_id, (row.slug, row.outcome));
            }
            Ok(out)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(HashMap::new()),
        Err(e) => Err(e).with_context(|| format!("open {}", path.display())),
    }
}

fn write_availability_cache(path: &Path, cache: &AvailabilityCache) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create availability cache dir {}", parent.display()))?;
    }
    let tmp = path.with_extension("tmp");
    let mut rows: Vec<_> = cache
        .iter()
        .map(|(asset_id, (slug, outcome))| AvailabilityCacheEntry {
            asset_id: asset_id.clone(),
            slug: slug.clone(),
            outcome: outcome.clone(),
        })
        .collect();
    rows.sort_by(|a, b| a.asset_id.cmp(&b.asset_id));

    let mut file =
        std::fs::File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
    for row in &rows {
        writeln!(file, "{}", serde_json::to_string(row)?)?;
    }
    file.sync_all()
        .with_context(|| format!("sync {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Parse `btc-updown-5m-1778587500` -> 1778587500.
pub fn parse_close_ts(slug: &str) -> Option<i64> {
    slug.rsplit('-').next().and_then(|t| t.parse::<i64>().ok())
}

async fn resolve_market_handles(
    asset_ids: Vec<String>,
    date: &str,
    slug_prefix: &str,
    max_concurrent: usize,
    availability_cache: Option<&Path>,
) -> Result<Vec<MarketHandle>> {
    let mut cache = match availability_cache {
        Some(path) => load_availability_cache(path)?,
        None => HashMap::new(),
    };

    let mut resolved = Vec::new();
    let mut missing = Vec::new();
    for asset_id in asset_ids {
        if let Some((slug, outcome)) = cache.get(&asset_id) {
            resolved.push((asset_id, slug.clone(), outcome.clone(), date.to_string()));
        } else {
            missing.push(asset_id);
        }
    }

    tracing::info!(
        date,
        cache_hits = resolved.len(),
        cache_misses = missing.len(),
        cache_path = ?availability_cache,
        "availability cache checked"
    );

    if !missing.is_empty() {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .context("build reqwest client")?;

        let mut fetched = futures::stream::iter(missing.into_iter().map(|asset_id| {
            let client = client.clone();
            let date = date.to_string();
            async move {
                match fetch_availability(&client, &asset_id).await {
                    Ok((slug, outcome)) => Some((asset_id, slug, outcome, date)),
                    Err(e) => {
                        tracing::warn!(asset = %asset_id, error = %e, "availability failed");
                        None
                    }
                }
            }
        }))
        .buffer_unordered(max_concurrent.max(1));

        let mut new_cache_entries = 0usize;
        while let Some(row) = fetched.next().await {
            let Some((asset_id, slug, outcome, row_date)) = row else {
                continue;
            };
            cache.insert(asset_id.clone(), (slug.clone(), outcome.clone()));
            resolved.push((asset_id, slug, outcome, row_date));
            new_cache_entries += 1;
            if new_cache_entries % 100 == 0 {
                if let Some(path) = availability_cache {
                    write_availability_cache(path, &cache)?;
                    tracing::info!(
                        date,
                        cached_entries = cache.len(),
                        new_cache_entries,
                        cache_path = ?path,
                        "availability cache checkpointed"
                    );
                }
            }
        }
    }

    if let Some(path) = availability_cache {
        write_availability_cache(path, &cache)?;
    }

    let mut out = Vec::new();
    for (asset_id, slug, outcome, date) in resolved {
        if !slug.starts_with(slug_prefix) {
            continue;
        }
        let Some(close_ts) = parse_close_ts(&slug) else {
            continue;
        };
        out.push(MarketHandle {
            asset_id,
            slug,
            close_ts,
            outcome,
            date,
        });
    }
    out.sort_by_key(|m| m.close_ts);
    Ok(out)
}

/// Discover all BTC 5min markets for the given date.
///
/// 1. Lists asset_ids under the S3 partition.
/// 2. Concurrently fetches slug+outcome from the Telonex availability API.
/// 3. Filters slugs to the BTC-updown-5m pattern (or whatever `slug_prefix` says).
/// 4. Parses the resolution timestamp from the slug.
pub async fn discover_markets(
    store: &TelonexStore,
    date: &str,
    slug_prefix: &str,
    max_concurrent: usize,
    availability_cache: Option<&Path>,
) -> Result<Vec<MarketHandle>> {
    let asset_ids = list_asset_ids_for_day(store, date).await?;
    tracing::info!(date, total_assets = asset_ids.len(), "S3 listing done");

    resolve_market_handles(
        asset_ids,
        date,
        slug_prefix,
        max_concurrent,
        availability_cache,
    )
    .await
}

/// Discover markets from local cached book partitions, resolving slug/outcome
/// through the Telonex availability API.
pub async fn discover_markets_from_local_cache(
    cache_dir: &Path,
    date: &str,
    slug_prefix: &str,
    max_concurrent: usize,
    max_assets: usize,
    availability_cache: Option<&Path>,
) -> Result<Vec<MarketHandle>> {
    let mut asset_ids = list_asset_ids_for_local_cache_day(cache_dir, date)?;
    if max_assets > 0 {
        asset_ids.truncate(max_assets);
    }
    tracing::info!(
        date,
        total_assets = asset_ids.len(),
        max_assets,
        cache_dir = ?cache_dir,
        "local cache listing done"
    );

    resolve_market_handles(
        asset_ids,
        date,
        slug_prefix,
        max_concurrent,
        availability_cache,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn lists_asset_ids_from_local_cache_day() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("pm-discovery-cache-test-{unique}"));
        let day_dir =
            root.join("raw/telonex/exchange=polymarket/channel=book_snapshot_25/date=2026-05-05");
        std::fs::create_dir_all(day_dir.join("asset_id=b")).expect("create asset b");
        std::fs::create_dir_all(day_dir.join("asset_id=a")).expect("create asset a");
        std::fs::write(day_dir.join("ignore.txt"), "").expect("write non-dir marker");

        let assets = list_asset_ids_for_local_cache_day(&root, "2026-05-05").expect("list assets");
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(assets, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn round_trips_availability_cache_jsonl() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("pm-availability-cache-test-{unique}"));
        let path = root.join("availability.jsonl");

        let mut cache = AvailabilityCache::new();
        cache.insert(
            "asset-b".to_string(),
            ("btc-updown-5m-1778587500".to_string(), "Up".to_string()),
        );
        cache.insert(
            "asset-a".to_string(),
            ("btc-updown-5m-1778587200".to_string(), "Down".to_string()),
        );

        write_availability_cache(&path, &cache).expect("write cache");
        let loaded = load_availability_cache(&path).expect("load cache");
        let _ = std::fs::remove_dir_all(&root);

        assert_eq!(loaded, cache);
    }
}
