use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use pm_strategy::bonereaper_v2::BonereaperV2GateStats;
use serde::{Deserialize, Serialize};

const MARKETS_PER_DAY: f64 = 288.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSummary {
    pub strategy: String,
    pub markets: usize,
    pub elapsed_days: f64,
    pub first_start_equity_usdc: f64,
    pub last_end_equity_usdc: f64,
    pub total_pnl_usdc: f64,
    pub compounded_return_pct: f64,
    pub compounded_daily_return_pct: f64,
    pub path_max_drawdown_pct: f64,
    pub orders_submitted: usize,
    pub orders_filled: usize,
    pub markets_with_fills: usize,
    pub winning_markets: usize,
    pub losing_markets: usize,
    pub hit_rate_filled_markets: f64,
    pub worst_market_slug: Option<String>,
    pub worst_market_pnl_usdc: f64,
    pub best_market_slug: Option<String>,
    pub best_market_pnl_usdc: f64,
    pub by_fill_tag: HashMap<String, FillTagSummary>,
    pub bonereaper_v2_gate_stats: Option<BonereaperV2GateStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FillTagSummary {
    pub fills: usize,
    pub total_notional_usdc: f64,
    pub avg_fill_price: f64,
    pub avg_side_edge_vs_fill: f64,
    pub avg_regime_whipsaw_score: f64,
    pub avg_regime_path_efficiency: f64,
    pub avg_regime_reversal_pressure: f64,
    pub avg_regime_sign_flip_rate: f64,
    pub avg_regime_realized_vol_180s_bps: f64,
}

#[derive(Debug, Default)]
struct FillTagAccumulator {
    fills: usize,
    total_notional_usdc: f64,
    sum_fill_price: f64,
    sum_side_edge_vs_fill: f64,
    side_edge_samples: usize,
    sum_regime_whipsaw_score: f64,
    sum_regime_path_efficiency: f64,
    sum_regime_reversal_pressure: f64,
    sum_regime_sign_flip_rate: f64,
    sum_regime_realized_vol_180s_bps: f64,
    regime_samples: usize,
}

impl FillTagAccumulator {
    fn push(&mut self, fill: &FillRow) {
        self.fills += 1;
        self.total_notional_usdc += fill.notional;
        self.sum_fill_price += fill.price;
        if let Some(edge) = fill.side_edge_vs_fill {
            self.sum_side_edge_vs_fill += edge;
            self.side_edge_samples += 1;
        }
        if let (
            Some(whipsaw),
            Some(path_efficiency),
            Some(reversal_pressure),
            Some(sign_flip_rate),
            Some(realized_vol),
        ) = (
            fill.regime_whipsaw_score,
            fill.regime_path_efficiency,
            fill.regime_reversal_pressure,
            fill.regime_sign_flip_rate,
            fill.regime_realized_vol_180s_bps,
        ) {
            self.sum_regime_whipsaw_score += whipsaw;
            self.sum_regime_path_efficiency += path_efficiency;
            self.sum_regime_reversal_pressure += reversal_pressure;
            self.sum_regime_sign_flip_rate += sign_flip_rate;
            self.sum_regime_realized_vol_180s_bps += realized_vol;
            self.regime_samples += 1;
        }
    }

    fn into_summary(self) -> FillTagSummary {
        FillTagSummary {
            fills: self.fills,
            total_notional_usdc: self.total_notional_usdc,
            avg_fill_price: if self.fills > 0 {
                self.sum_fill_price / self.fills as f64
            } else {
                0.0
            },
            avg_side_edge_vs_fill: if self.side_edge_samples > 0 {
                self.sum_side_edge_vs_fill / self.side_edge_samples as f64
            } else {
                0.0
            },
            avg_regime_whipsaw_score: if self.regime_samples > 0 {
                self.sum_regime_whipsaw_score / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_path_efficiency: if self.regime_samples > 0 {
                self.sum_regime_path_efficiency / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_reversal_pressure: if self.regime_samples > 0 {
                self.sum_regime_reversal_pressure / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_sign_flip_rate: if self.regime_samples > 0 {
                self.sum_regime_sign_flip_rate / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_realized_vol_180s_bps: if self.regime_samples > 0 {
                self.sum_regime_realized_vol_180s_bps / self.regime_samples as f64
            } else {
                0.0
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct MarketResultRow {
    slug: String,
    per_strategy: HashMap<String, StrategyResultRow>,
}

#[derive(Debug, Deserialize)]
struct StrategyResultRow {
    #[serde(default)]
    orders_submitted: usize,
    #[serde(default)]
    orders_filled: usize,
    #[serde(default)]
    pnl_usdc: f64,
    #[serde(default)]
    start_equity_usdc: f64,
    #[serde(default)]
    end_equity_usdc: f64,
    #[serde(default)]
    fills_detail: Vec<FillRow>,
    #[serde(default)]
    bonereaper_v2_gate_stats: Option<BonereaperV2GateStats>,
}

#[derive(Debug, Deserialize)]
struct FillRow {
    #[serde(default)]
    price: f64,
    #[serde(default)]
    notional: f64,
    #[serde(default)]
    tag: String,
    #[serde(default)]
    side_edge_vs_fill: Option<f64>,
    #[serde(default)]
    regime_whipsaw_score: Option<f64>,
    #[serde(default)]
    regime_path_efficiency: Option<f64>,
    #[serde(default)]
    regime_reversal_pressure: Option<f64>,
    #[serde(default)]
    regime_sign_flip_rate: Option<f64>,
    #[serde(default)]
    regime_realized_vol_180s_bps: Option<f64>,
}

pub fn summarize_markets_jsonl(path: &Path, strategy: &str) -> Result<ResultSummary> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let reader = BufReader::new(file);

    let mut markets = 0usize;
    let mut first_start_equity = None::<f64>;
    let mut last_end_equity = 0.0f64;
    let mut peak_equity = 0.0f64;
    let mut path_max_drawdown = 0.0f64;
    let mut total_pnl = 0.0f64;
    let mut orders_submitted = 0usize;
    let mut orders_filled = 0usize;
    let mut markets_with_fills = 0usize;
    let mut winning_markets = 0usize;
    let mut losing_markets = 0usize;
    let mut worst_market_slug = None::<String>;
    let mut worst_market_pnl = 0.0f64;
    let mut best_market_slug = None::<String>;
    let mut best_market_pnl = 0.0f64;
    let mut tag_acc: HashMap<String, FillTagAccumulator> = HashMap::new();
    let mut bonereaper_v2_gate_stats = None::<BonereaperV2GateStats>;

    for (line_no, line) in reader.lines().enumerate() {
        let line = line.with_context(|| format!("read line {}", line_no + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        let row: MarketResultRow = serde_json::from_str(&line)
            .with_context(|| format!("parse {} line {}", path.display(), line_no + 1))?;
        let Some(strategy_row) = row.per_strategy.get(strategy) else {
            continue;
        };

        markets += 1;
        first_start_equity.get_or_insert(strategy_row.start_equity_usdc);
        last_end_equity = strategy_row.end_equity_usdc;
        peak_equity = peak_equity.max(last_end_equity);
        if peak_equity > 0.0 {
            path_max_drawdown =
                path_max_drawdown.max((peak_equity - last_end_equity) / peak_equity);
        }
        total_pnl += strategy_row.pnl_usdc;
        orders_submitted += strategy_row.orders_submitted;
        orders_filled += strategy_row.orders_filled;
        if strategy_row.orders_filled > 0 {
            markets_with_fills += 1;
        }
        if strategy_row.pnl_usdc > 0.0 {
            winning_markets += 1;
        } else if strategy_row.pnl_usdc < 0.0 {
            losing_markets += 1;
        }
        if strategy_row.pnl_usdc < worst_market_pnl {
            worst_market_pnl = strategy_row.pnl_usdc;
            worst_market_slug = Some(row.slug.clone());
        }
        if strategy_row.pnl_usdc > best_market_pnl {
            best_market_pnl = strategy_row.pnl_usdc;
            best_market_slug = Some(row.slug);
        }
        for fill in &strategy_row.fills_detail {
            let tag = if fill.tag.is_empty() {
                "unknown".to_string()
            } else {
                fill.tag.clone()
            };
            tag_acc.entry(tag).or_default().push(fill);
        }
        if let Some(stats) = strategy_row.bonereaper_v2_gate_stats {
            bonereaper_v2_gate_stats
                .get_or_insert_with(BonereaperV2GateStats::default)
                .add_assign(stats);
        }
    }

    let first_start_equity = first_start_equity.unwrap_or(0.0);
    let elapsed_days = markets as f64 / MARKETS_PER_DAY;
    let compounded_return_pct = if first_start_equity > 0.0 {
        (last_end_equity / first_start_equity - 1.0) * 100.0
    } else {
        0.0
    };
    let compounded_daily_return_pct =
        if elapsed_days > 0.0 && first_start_equity > 0.0 && last_end_equity > 0.0 {
            ((last_end_equity / first_start_equity).powf(1.0 / elapsed_days) - 1.0) * 100.0
        } else {
            0.0
        };
    let hit_rate_filled_markets = if markets_with_fills > 0 {
        winning_markets as f64 / markets_with_fills as f64
    } else {
        0.0
    };
    let by_fill_tag = tag_acc
        .into_iter()
        .map(|(tag, acc)| (tag, acc.into_summary()))
        .collect();

    Ok(ResultSummary {
        strategy: strategy.to_string(),
        markets,
        elapsed_days,
        first_start_equity_usdc: first_start_equity,
        last_end_equity_usdc: last_end_equity,
        total_pnl_usdc: total_pnl,
        compounded_return_pct,
        compounded_daily_return_pct,
        path_max_drawdown_pct: path_max_drawdown * 100.0,
        orders_submitted,
        orders_filled,
        markets_with_fills,
        winning_markets,
        losing_markets,
        hit_rate_filled_markets,
        worst_market_slug,
        worst_market_pnl_usdc: worst_market_pnl,
        best_market_slug,
        best_market_pnl_usdc: best_market_pnl,
        by_fill_tag,
        bonereaper_v2_gate_stats,
    })
}

pub fn write_result_summary_json(path: &Path, summary: &ResultSummary) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    std::fs::write(path, serde_json::to_string_pretty(summary)?)
        .with_context(|| format!("write {}", path.display()))
}

pub fn print_result_summary(summary: &ResultSummary) {
    println!("== market-results summary ==");
    println!("strategy              : {}", summary.strategy);
    println!(
        "markets / days        : {} / {:.1}",
        summary.markets, summary.elapsed_days
    );
    println!(
        "equity / pnl          : {:.2} / {:+.2}",
        summary.last_end_equity_usdc, summary.total_pnl_usdc
    );
    println!(
        "return / daily        : {:+.2}% / {:+.3}%",
        summary.compounded_return_pct, summary.compounded_daily_return_pct
    );
    println!(
        "max drawdown          : {:.2}%",
        summary.path_max_drawdown_pct
    );
    println!(
        "orders / fills        : {} / {}",
        summary.orders_submitted, summary.orders_filled
    );
    println!(
        "fill markets W/L      : {} / {} / {} ({:.1}% hit)",
        summary.markets_with_fills,
        summary.winning_markets,
        summary.losing_markets,
        summary.hit_rate_filled_markets * 100.0
    );
    println!(
        "worst market          : {:+.2} {}",
        summary.worst_market_pnl_usdc,
        summary.worst_market_slug.as_deref().unwrap_or("-")
    );
    println!(
        "best market           : {:+.2} {}",
        summary.best_market_pnl_usdc,
        summary.best_market_slug.as_deref().unwrap_or("-")
    );
    if !summary.by_fill_tag.is_empty() {
        println!("fills by tag:");
        let mut tags: Vec<_> = summary.by_fill_tag.iter().collect();
        tags.sort_by(|a, b| a.0.cmp(b.0));
        for (tag, agg) in tags {
            println!(
                "  {:28} fills={:<6} notional={:.2} avg_px={:.4} edge_fill={:+.4} whip={:.3} path={:.3} rev={:.3}",
                tag,
                agg.fills,
                agg.total_notional_usdc,
                agg.avg_fill_price,
                agg.avg_side_edge_vs_fill,
                agg.avg_regime_whipsaw_score,
                agg.avg_regime_path_efficiency,
                agg.avg_regime_reversal_pressure
            );
        }
    }
    if let Some(stats) = summary.bonereaper_v2_gate_stats {
        println!("late favourite gates:");
        println!(
            "  checks={} emits={} skew_fail={} sustain_fail={} price_fail={} whipsaw_fail={}",
            stats.late_favourite_checks,
            stats.late_favourite_emits,
            stats.late_favourite_skew_fail,
            stats.late_favourite_sustain_fail,
            stats.late_favourite_price_fail,
            stats.late_favourite_whipsaw_fail
        );
        println!(
            "  model_conf_fail={} model_risk_fail={} model_side_p_fail={} model_dir_fail={} reversal_fail={} path_eff_fail={} adverse_mom_fail={} pullback_fail={} avg_entry_dd_fail={}",
            stats.late_favourite_model_confidence_fail,
            stats.late_favourite_model_risk_fail,
            stats.late_favourite_model_side_p_fail,
            stats.late_favourite_model_direction_fail,
            stats.late_favourite_reversal_pressure_fail,
            stats.late_favourite_path_efficiency_fail,
            stats.late_favourite_adverse_momentum_fail,
            stats.late_favourite_entry_pullback_fail,
            stats.late_favourite_avg_entry_drawdown_fail
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn summarizes_compounded_portfolio_results() {
        let path = std::env::temp_dir().join(format!(
            "pm-result-summary-test-{}.jsonl",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let mut file = File::create(&path).unwrap();
        writeln!(
            file,
            r#"{{"slug":"m1","per_strategy":{{"bonereaper_v2":{{"orders_submitted":1,"orders_filled":1,"pnl_usdc":10.0,"start_equity_usdc":1000.0,"end_equity_usdc":1010.0,"fills_detail":[{{"price":0.8,"notional":8.0,"tag":"fav"}}],"bonereaper_v2_gate_stats":{{"late_favourite_checks":10,"late_favourite_emits":1,"late_favourite_whipsaw_fail":2}}}}}}}}"#
        )
        .unwrap();
        writeln!(
            file,
            r#"{{"slug":"m2","per_strategy":{{"bonereaper_v2":{{"orders_submitted":1,"orders_filled":1,"pnl_usdc":-20.0,"start_equity_usdc":1010.0,"end_equity_usdc":990.0,"fills_detail":[{{"price":0.9,"notional":9.0,"tag":"fav"}}],"bonereaper_v2_gate_stats":{{"late_favourite_checks":5,"late_favourite_emits":2,"late_favourite_reversal_pressure_fail":3}}}}}}}}"#
        )
        .unwrap();
        drop(file);

        let summary = summarize_markets_jsonl(&path, "bonereaper_v2").unwrap();
        let _ = std::fs::remove_file(&path);
        assert_eq!(summary.markets, 2);
        assert_eq!(summary.orders_filled, 2);
        assert_eq!(summary.markets_with_fills, 2);
        assert_eq!(summary.winning_markets, 1);
        assert_eq!(summary.losing_markets, 1);
        assert_eq!(summary.worst_market_slug.as_deref(), Some("m2"));
        assert!((summary.path_max_drawdown_pct - 1.9801980198019802).abs() < 1e-9);
        let fav = summary.by_fill_tag.get("fav").unwrap();
        assert_eq!(fav.fills, 2);
        assert!((fav.total_notional_usdc - 17.0).abs() < 1e-9);
        assert!((fav.avg_fill_price - 0.85).abs() < 1e-9);
        let gates = summary.bonereaper_v2_gate_stats.unwrap();
        assert_eq!(gates.late_favourite_checks, 15);
        assert_eq!(gates.late_favourite_emits, 3);
        assert_eq!(gates.late_favourite_whipsaw_fail, 2);
        assert_eq!(gates.late_favourite_reversal_pressure_fail, 3);
    }
}
