use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use futures::StreamExt;
use object_store::{
    ObjectStore, aws::AmazonS3Builder, local::LocalFileSystem, path::Path as ObjectPath,
};

use crate::schema::Channel;

/// Configuration for connecting to the Telonex S3 mirror.
///
/// The binary never mutates env. Credentials come from the standard AWS chain
/// (env vars locally; IAM role on EC2/ECS). For local dev under an SSO profile
/// run once per shell:
///
///   eval "$(aws configure export-credentials --profile visumlabs --format env)"
#[derive(Debug, Clone)]
pub struct TelonexStoreConfig {
    pub bucket: String,
    pub region: String,
}

impl TelonexStoreConfig {
    pub fn from_env() -> Result<Self> {
        let bucket = std::env::var("PM_TELONEX_BUCKET")
            .unwrap_or_else(|_| "pm-research-data-prod".to_string());
        let region = std::env::var("PM_TELONEX_REGION")
            .or_else(|_| std::env::var("AWS_REGION"))
            .or_else(|_| std::env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "eu-west-2".to_string());
        Ok(Self { bucket, region })
    }
}

/// Wrapper around an `object_store::aws::AmazonS3` instance. Cheap to clone.
#[derive(Clone)]
pub struct TelonexStore {
    pub bucket: String,
    inner: Arc<dyn ObjectStore>,
}

impl TelonexStore {
    pub fn try_new(cfg: &TelonexStoreConfig) -> Result<Self> {
        let builder = AmazonS3Builder::from_env()
            .with_bucket_name(&cfg.bucket)
            .with_region(&cfg.region);
        let inner = builder.build().with_context(|| {
            format!(
                "build S3 client for bucket={} region={} \
                 (creds: env / IAM role; for SSO run \
                 `eval \"$(aws configure export-credentials --profile <name> --format env)\"`)",
                cfg.bucket, cfg.region
            )
        })?;
        Ok(Self {
            bucket: cfg.bucket.clone(),
            inner: Arc::new(inner),
        })
    }

    /// Local-filesystem-backed store rooted at `root_dir`. The directory layout
    /// mirrors the S3 prefix structure so paths used by the rest of the loader
    /// (e.g. `raw/telonex/exchange=polymarket/...`) remain unchanged.
    ///
    /// Used by `prep-cache` to write parquets and by backtest runs with
    /// `--local-cache-dir` to read them mmap-style off local disk.
    pub fn try_new_local(root_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&root_dir)
            .with_context(|| format!("mkdir -p {}", root_dir.display()))?;
        let fs = LocalFileSystem::new_with_prefix(&root_dir)
            .with_context(|| format!("LocalFileSystem at {}", root_dir.display()))?;
        Ok(Self {
            bucket: format!("local:{}", root_dir.display()),
            inner: Arc::new(fs),
        })
    }

    pub fn store(&self) -> Arc<dyn ObjectStore> {
        self.inner.clone()
    }

    /// List all objects under a prefix. Returns at most `limit` paths (sorted).
    pub async fn list_prefix(&self, prefix: &ObjectPath, limit: usize) -> Result<Vec<ObjectPath>> {
        let mut stream = self.inner.list(Some(prefix));
        let mut out = Vec::new();
        while let Some(meta) = stream.next().await {
            let meta = meta.with_context(|| format!("list {prefix}"))?;
            out.push(meta.location);
            if out.len() >= limit {
                break;
            }
        }
        out.sort();
        Ok(out)
    }

    /// Resolve the parquet file path for one (channel, date, asset_id) cell.
    ///
    /// Telonex partitions are Hive-style:
    ///   raw/telonex/exchange={ex}/channel={ch}/date={d}/asset_id={a}/<file>.parquet
    ///
    /// There is normally exactly one file in that leaf prefix.
    pub async fn resolve_asset_day(
        &self,
        exchange: &str,
        channel: Channel,
        date: &str,
        asset_id: &str,
    ) -> Result<ObjectPath> {
        let prefix = build_object_path(exchange, channel, date, asset_id);
        let files = self.list_prefix(&prefix, 8).await?;
        files
            .into_iter()
            .find(|p| p.as_ref().ends_with(".parquet"))
            .ok_or_else(|| anyhow!("no .parquet under {prefix}"))
    }
}

/// Build the Hive-partitioned prefix used by Telonex on S3.
pub fn build_object_path(
    exchange: &str,
    channel: Channel,
    date: &str,
    asset_id: &str,
) -> ObjectPath {
    let raw = format!(
        "raw/telonex/exchange={exchange}/channel={channel}/date={date}/asset_id={asset_id}/",
        channel = channel.as_str()
    );
    ObjectPath::from(raw)
}
