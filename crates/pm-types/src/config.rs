use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BacktestMode {
    Fast,
    Portfolio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    pub date: String,
    pub mode: BacktestMode,
    pub data_dir: PathBuf,
    pub starting_equity_usdc: f64,
    pub max_open_risk_usdc: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub backtest: BacktestConfig,
}
