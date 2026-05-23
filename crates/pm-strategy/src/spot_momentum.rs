//! Multi-timeframe spot momentum.
//!
//! Ported with light modifications from `polymarket-exec/src/signals/momentum.rs`.
//! Drives the `direction_score` component of the 4-score model.
//!
//! Inputs are `SpotHistory` samples (ns timestamps + USD price). Outputs a
//! signed score in `[-1, 1]` plus per-window returns in basis points and an
//! acceleration term (latest minus prior window).

use pm_types::SpotHistory;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SignalDirection {
    Up,
    Down,
    #[default]
    Neutral,
}

impl SignalDirection {
    pub fn from_signed(value: f64, deadband: f64) -> Self {
        if !value.is_finite() || value.abs() <= deadband {
            Self::Neutral
        } else if value > 0.0 {
            Self::Up
        } else {
            Self::Down
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MomentumConfig {
    pub lookback_windows: usize,
    pub window_ns: i64,
    pub decay_factor: f64,
    pub directional_deadband: f64,
}

impl Default for MomentumConfig {
    fn default() -> Self {
        Self {
            lookback_windows: 5,
            window_ns: 30 * 1_000_000_000, // 30 seconds
            decay_factor: 0.6,
            directional_deadband: 0.10,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MomentumSignal {
    pub direction: SignalDirection,
    /// Signed weighted score in roughly `[-1, 1]` (sum of decayed signs).
    pub score: f64,
    /// `|score|` clipped to `[0, 1]`.
    pub strength: f64,
    pub latest_window_return_bps: Option<f64>,
    pub acceleration_bps: Option<f64>,
    pub window_returns_bps: Vec<f64>,
}

impl MomentumSignal {
    pub fn is_ready(&self) -> bool {
        !self.window_returns_bps.is_empty()
    }
}

/// Compute multi-timeframe momentum over the trailing windows ending at `now_ns`.
pub fn compute_momentum(
    now_ns: i64,
    spot: &SpotHistory,
    cfg: &MomentumConfig,
) -> MomentumSignal {
    if cfg.window_ns <= 0 || cfg.lookback_windows == 0 || spot.is_empty() {
        return MomentumSignal::default();
    }
    let mut returns = Vec::with_capacity(cfg.lookback_windows);
    for window_idx in 0..cfg.lookback_windows {
        let end_ns = now_ns - (window_idx as i64) * cfg.window_ns;
        let start_ns = end_ns - cfg.window_ns;
        let Some(start_price) = spot.price_at_or_after(start_ns) else { break };
        let Some(end_price) = spot.price_at_or_before(end_ns) else { break };
        if start_price <= 0.0 || end_price <= 0.0 {
            break;
        }
        returns.push(((end_price / start_price) - 1.0) * 10_000.0);
    }
    if returns.is_empty() {
        return MomentumSignal::default();
    }
    let mut weighted = 0.0f64;
    for (i, ret) in returns.iter().enumerate() {
        let w = cfg.decay_factor.clamp(0.0, 1.0).powi(i as i32);
        weighted += ret.signum() * w;
    }
    let latest = returns.first().copied();
    let accel = returns
        .first()
        .zip(returns.get(1))
        .map(|(a, b)| a - b);
    MomentumSignal {
        direction: SignalDirection::from_signed(weighted, cfg.directional_deadband),
        score: weighted,
        strength: weighted.abs().clamp(0.0, 1.0),
        latest_window_return_bps: latest,
        acceleration_bps: accel,
        window_returns_bps: returns,
    }
}

/// Weighted multi-timeframe return (the canonical "spot momentum"
/// number the spec describes). Returns are in raw fractional returns
/// (e.g. 0.001 = 10 bps) so callers don't have to keep converting.
///
/// Weights are the spec values: 10s/30s/60s/120s/300s = 0.45/0.25/0.15/0.10/0.05.
pub fn weighted_multi_tf_return(now_ns: i64, spot: &SpotHistory) -> Option<f64> {
    let s = |secs: i64| spot.simple_return(now_ns, secs * 1_000_000_000);
    let r10 = s(10);
    let r30 = s(30);
    let r60 = s(60);
    let r120 = s(120);
    let r300 = s(300);
    if r10.is_none() && r30.is_none() && r60.is_none() && r120.is_none() && r300.is_none() {
        return None;
    }
    let mut sum = 0.0f64;
    let mut wsum = 0.0f64;
    if let Some(r) = r10 { sum += 0.45 * r; wsum += 0.45; }
    if let Some(r) = r30 { sum += 0.25 * r; wsum += 0.25; }
    if let Some(r) = r60 { sum += 0.15 * r; wsum += 0.15; }
    if let Some(r) = r120 { sum += 0.10 * r; wsum += 0.10; }
    if let Some(r) = r300 { sum += 0.05 * r; wsum += 0.05; }
    if wsum <= 0.0 {
        return None;
    }
    Some(sum / wsum)
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
    fn downtrend_produces_down_signal() {
        let ns = |secs: i64| secs * 1_000_000_000;
        let spot = h(vec![
            (ns(0), 100.0),
            (ns(30), 99.0),
            (ns(60), 98.0),
            (ns(90), 97.0),
        ]);
        let sig = compute_momentum(
            ns(90),
            &spot,
            &MomentumConfig {
                lookback_windows: 3,
                window_ns: ns(30),
                decay_factor: 0.6,
                directional_deadband: 0.05,
            },
        );
        assert_eq!(sig.direction, SignalDirection::Down);
        assert!(sig.score < -0.5, "score {}", sig.score);
    }

    #[test]
    fn weighted_multi_tf_uptrend_positive() {
        let ns = |secs: i64| secs * 1_000_000_000;
        // 1% rise over 5 min, mostly in the last 60s.
        let spot = h(vec![
            (ns(0), 80_000.0),
            (ns(60), 80_100.0),
            (ns(120), 80_200.0),
            (ns(240), 80_300.0),
            (ns(290), 80_500.0),
            (ns(300), 80_800.0),
        ]);
        let r = weighted_multi_tf_return(ns(300), &spot).unwrap();
        assert!(r > 0.0, "got {r}");
    }
}
