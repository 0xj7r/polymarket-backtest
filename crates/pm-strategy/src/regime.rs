//! BTC micro-regime classifier.
//!
//! Ported from `polymarket-exec/src/signals/btc_regime.rs`. Builds a snapshot
//! from a `SpotHistory` at a given `now_ns` and classifies the regime into one
//! of four quadrants on (vol-level × trend-strength).

use pm_types::SpotHistory;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BtcRegime {
    /// Low noise, no trend. Skip everything; nothing to trade.
    Flat,
    /// High noise, no trend. Mean-reverting; paired-MM has positive edge.
    Whipsaw,
    /// Low noise, strong trend. Late-bar directional has positive edge.
    DirectionalSmooth,
    /// High noise + strong trend. Mixed; both lanes cautious.
    TrendingVolatile,
}

impl BtcRegime {
    pub const VOL_LOW_HIGH_BPS: f64 = 8.0;
    pub const TREND_NOISE_RATIO: f64 = 5.0;
    pub const VOL_FLOOR_BPS: f64 = 0.5;
}

impl std::fmt::Display for BtcRegime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BtcRegime::Flat => f.write_str("flat"),
            BtcRegime::Whipsaw => f.write_str("whipsaw"),
            BtcRegime::DirectionalSmooth => f.write_str("directional_smooth"),
            BtcRegime::TrendingVolatile => f.write_str("trending_volatile"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BtcRegimeSnapshot {
    pub last_price: Option<f64>,
    pub price_history_ms: u64,
    pub realized_vol_5m_bps: Option<f64>,
    pub realized_vol_15m_bps: Option<f64>,
    pub trade_count_5m: u64,
    pub trade_count_15m: u64,
    pub return_30s_bps: Option<f64>,
    pub return_60s_bps: Option<f64>,
    pub return_120s_bps: Option<f64>,
    pub return_180s_bps: Option<f64>,
    pub observed_at_ns: i64,
}

impl BtcRegimeSnapshot {
    /// Build from a spot tape at `now_ns`. Returns a snapshot whose individual
    /// fields may be `None` during warmup.
    pub fn from_history(now_ns: i64, spot: &SpotHistory) -> Self {
        let last_price = spot.price_at_or_before(now_ns);
        let first_ts = spot.samples().first().map(|t| t.ts_ns);
        let price_history_ms = first_ts
            .map(|t| ((now_ns - t).max(0) / 1_000_000) as u64)
            .unwrap_or(0);

        let return_bps = |sec: i64| {
            spot.simple_return(now_ns, sec * 1_000_000_000)
                .map(|r| r * 10_000.0)
        };
        let return_30s_bps = return_bps(30);
        let return_60s_bps = return_bps(60);
        let return_120s_bps = return_bps(120);
        let return_180s_bps = return_bps(180);

        let (realized_vol_5m_bps, trade_count_5m) = realized_vol_window(now_ns, spot, 300);
        let (realized_vol_15m_bps, trade_count_15m) = realized_vol_window(now_ns, spot, 900);

        Self {
            last_price,
            price_history_ms,
            realized_vol_5m_bps,
            realized_vol_15m_bps,
            trade_count_5m,
            trade_count_15m,
            return_30s_bps,
            return_60s_bps,
            return_120s_bps,
            return_180s_bps,
            observed_at_ns: now_ns,
        }
    }

    /// Classify the regime. `None` during warmup (either input unavailable).
    pub fn regime(&self) -> Option<BtcRegime> {
        let vol = self
            .realized_vol_5m_bps
            .filter(|v| v.is_finite() && *v >= 0.0)?;
        let trend = self
            .return_180s_bps
            .filter(|r| r.is_finite())
            .map(f64::abs)?;
        let vol_eff = vol.max(BtcRegime::VOL_FLOOR_BPS);
        let trend_dominates = (trend / vol_eff) >= BtcRegime::TREND_NOISE_RATIO;
        let high_vol = vol >= BtcRegime::VOL_LOW_HIGH_BPS;
        Some(match (high_vol, trend_dominates) {
            (false, false) => BtcRegime::Flat,
            (false, true) => BtcRegime::DirectionalSmooth,
            (true, false) => BtcRegime::Whipsaw,
            (true, true) => BtcRegime::TrendingVolatile,
        })
    }
}

/// Realized vol (bps) of per-trade log returns over the trailing `secs`
/// window. Returns `(vol_bps, trade_count)`.
fn realized_vol_window(now_ns: i64, spot: &SpotHistory, secs: i64) -> (Option<f64>, u64) {
    let start_ns = now_ns - secs * 1_000_000_000;
    let slice = spot.range(start_ns, now_ns);
    if slice.len() < 3 {
        return (None, slice.len() as u64);
    }
    let mut prev = slice[0].price;
    let mut sumsq = 0.0f64;
    let mut n = 0usize;
    for t in &slice[1..] {
        let p = t.price;
        if p > 0.0 && prev > 0.0 && p.is_finite() {
            let r = (p / prev).ln();
            sumsq += r * r;
            n += 1;
        }
        prev = p;
    }
    if n < 2 {
        return (None, slice.len() as u64);
    }
    // RMS of log returns, converted to bps.
    let rms = (sumsq / n as f64).sqrt();
    (Some(rms * 10_000.0), slice.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pm_types::SpotTick;

    fn h(samples: Vec<(i64, f64)>) -> SpotHistory {
        SpotHistory::new(
            samples
                .into_iter()
                .map(|(ts_ns, price)| SpotTick {
                    ts_ns,
                    price,
                    quantity: 0.0,
                    is_buyer_maker: false,
                })
                .collect(),
        )
    }

    #[test]
    fn flat_regime_when_calm_and_no_trend() {
        let ns = |secs: i64| secs * 1_000_000_000;
        let mut samples = Vec::new();
        // 60 prices clustered tightly around 80000, every 5 seconds.
        for i in 0..60i64 {
            samples.push((ns(i * 5), 80_000.0 + (i as f64 % 2.0)));
        }
        let snap = BtcRegimeSnapshot::from_history(ns(300), &h(samples));
        assert_eq!(snap.regime(), Some(BtcRegime::Flat));
    }

    #[test]
    fn directional_smooth_when_steady_trend() {
        let ns = |secs: i64| secs * 1_000_000_000;
        let mut samples = Vec::new();
        // Smooth 20 bps drift up every 5 seconds over 5 min. Low vol, large trend.
        for i in 0..60i64 {
            samples.push((ns(i * 5), 80_000.0 + (i as f64) * 1.0));
        }
        let snap = BtcRegimeSnapshot::from_history(ns(300), &h(samples));
        // 60 * 1 / 80000 = 0.00075 = 75 bps move; vol is small (smooth).
        // Should fall into DirectionalSmooth, not Flat.
        assert!(matches!(
            snap.regime(),
            Some(BtcRegime::DirectionalSmooth) | Some(BtcRegime::TrendingVolatile)
        ));
    }
}
