//! Market discovery: list available asset_ids under a date prefix in S3 and
//! resolve their slug + resolution timestamp via the Telonex availability API.
//!
//! Output is JSONL of `MarketHandle` rows that `walk-forward` reads.

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use object_store::path::Path as ObjectPath;
use pm_telonex_loader::TelonexStore;
use serde::{Deserialize, Serialize};

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

/// Parse `btc-updown-5m-1778587500` -> 1778587500.
pub fn parse_close_ts(slug: &str) -> Option<i64> {
    slug.rsplit('-').next().and_then(|t| t.parse::<i64>().ok())
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
) -> Result<Vec<MarketHandle>> {
    let asset_ids = list_asset_ids_for_day(store, date).await?;
    tracing::info!(date, total_assets = asset_ids.len(), "S3 listing done");

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .context("build reqwest client")?;

    let results = futures::stream::iter(asset_ids.into_iter().map(|asset_id| {
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
    .buffer_unordered(max_concurrent.max(1))
    .collect::<Vec<_>>()
    .await;

    let mut out = Vec::new();
    for r in results.into_iter().flatten() {
        let (asset_id, slug, outcome, date) = r;
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
