//! Spot tape — Binance aggTrades (or equivalent) of the underlying asset.
//!
//! Strategies query `SpotHistory` for last-price lookups and rolling-window
//! returns. The history is sorted by `ts_ns` so all lookups are `O(log n)`
//! binary search.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SpotTick {
    pub ts_ns: i64,
    pub price: f64,
    pub quantity: f32,
    /// Binance aggTrade convention: `true` if the BUYER is the maker (i.e. the
    /// trade was initiated by a seller hitting the bid).
    pub is_buyer_maker: bool,
}

#[derive(Debug, Default, Clone)]
pub struct SpotHistory {
    samples: Vec<SpotTick>,
}

impl SpotHistory {
    pub fn new(mut samples: Vec<SpotTick>) -> Self {
        samples.sort_by_key(|t| t.ts_ns);
        Self { samples }
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn samples(&self) -> &[SpotTick] {
        &self.samples
    }

    /// Most recent price at or before `ts_ns`. `None` if no samples exist yet
    /// or the history starts after `ts_ns`.
    pub fn price_at_or_before(&self, ts_ns: i64) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let idx = match self.samples.binary_search_by_key(&ts_ns, |t| t.ts_ns) {
            Ok(i) => i,
            Err(0) => return None,
            Err(i) => i - 1,
        };
        Some(self.samples[idx].price)
    }

    /// First price at or after `ts_ns`. `None` if no later samples exist.
    pub fn price_at_or_after(&self, ts_ns: i64) -> Option<f64> {
        if self.samples.is_empty() {
            return None;
        }
        let idx = match self.samples.binary_search_by_key(&ts_ns, |t| t.ts_ns) {
            Ok(i) => i,
            Err(i) if i >= self.samples.len() => return None,
            Err(i) => i,
        };
        Some(self.samples[idx].price)
    }

    /// `(end - start) / start` simple return over the trailing `lookback_ns`
    /// window ending at `ts_ns`. `None` if either endpoint lacks data.
    pub fn simple_return(&self, ts_ns: i64, lookback_ns: i64) -> Option<f64> {
        let end_price = self.price_at_or_before(ts_ns)?;
        let start_price = self.price_at_or_after(ts_ns.saturating_sub(lookback_ns))?;
        if start_price <= 0.0 || !start_price.is_finite() {
            return None;
        }
        Some((end_price / start_price) - 1.0)
    }

    /// Indices of samples in `[start_ns, end_ns]` (inclusive). Useful for
    /// realized-vol calculations.
    pub fn range(&self, start_ns: i64, end_ns: i64) -> &[SpotTick] {
        let lo = self.samples.partition_point(|t| t.ts_ns < start_ns);
        let hi = self.samples.partition_point(|t| t.ts_ns <= end_ns);
        &self.samples[lo..hi]
    }

    /// Signed aggressor-flow statistics over the trailing `window_ns` window
    /// ending at `ts_ns` (inclusive `[ts_ns - window_ns, ts_ns]`), oriented to a
    /// load that is buying `yes` when `is_buy_yes` is true.
    ///
    /// Mirrors `_signed_flow_and_adverse` in
    /// `scripts/binance_flow_reversal_discovery.py`:
    /// `is_buyer_maker == true` is a seller-initiated (aggressive sell) print, so
    /// it accrues to sell volume; otherwise it is an aggressive buy. The flow
    /// imbalance is `(buy - sell) / (buy + sell)` in `[-1, +1]`, the adverse
    /// volume is the aggressor flow against the load side (sell volume for a
    /// BuyYes load, buy volume for a BuyNo load), the large-adverse count tallies
    /// adverse-side prints with quantity at least `LARGE_PRINT_QTY`, and the
    /// intensity is trades-per-second over the window.
    pub fn signed_flow_and_adverse(
        &self,
        ts_ns: i64,
        window_ns: i64,
        is_buy_yes: bool,
    ) -> FlowStats {
        const LARGE_PRINT_QTY: f32 = 50.0;
        let start = ts_ns.saturating_sub(window_ns);
        let window = self.range(start, ts_ns);

        let mut vol_buy = 0.0_f64;
        let mut vol_sell = 0.0_f64;
        let mut large_adverse = 0_u32;
        let n = window.len();
        for tick in window {
            let q = tick.quantity as f64;
            if tick.is_buyer_maker {
                vol_sell += q;
            } else {
                vol_buy += q;
            }
            let is_adverse = if is_buy_yes {
                tick.is_buyer_maker
            } else {
                !tick.is_buyer_maker
            };
            if is_adverse && tick.quantity >= LARGE_PRINT_QTY {
                large_adverse += 1;
            }
        }

        let total = vol_buy + vol_sell;
        let imbalance = if total > 1e-9 {
            (vol_buy - vol_sell) / total
        } else {
            0.0
        };
        let adverse_volume = if is_buy_yes { vol_sell } else { vol_buy };
        let intensity = if window_ns > 0 {
            n as f64 / (window_ns as f64 / 1e9)
        } else {
            0.0
        };

        FlowStats {
            imbalance,
            adverse_volume,
            large_adverse_count: large_adverse,
            intensity,
        }
    }

    /// Simple return over the trailing `lookback_ns` window using the last price
    /// at-or-before `ts_ns` against the last price at-or-before
    /// `ts_ns - lookback_ns`.
    ///
    /// This intentionally differs from [`Self::simple_return`] (which anchors the
    /// start at-or-after the window open). It matches `ret_at` in
    /// `scripts/binance_flow_reversal_discovery.py`, which uses at-or-before for
    /// both endpoints.
    pub fn trailing_return(&self, ts_ns: i64, lookback_ns: i64) -> Option<f64> {
        let end_price = self.price_at_or_before(ts_ns)?;
        let start_price = self.price_at_or_before(ts_ns.saturating_sub(lookback_ns))?;
        if start_price <= 0.0 || !start_price.is_finite() {
            return None;
        }
        Some((end_price / start_price) - 1.0)
    }

    /// Short-horizon spot returns (5/15/30s) and the two acceleration proxies
    /// used by the reversal discovery. Mirrors `_spot_returns_and_accel`: missing
    /// returns collapse to `0.0`, `accel_15s_vs_30s = (r15 - r30) / 15`, and
    /// `accel_5s_vs_15s = (r5 - r15) / 5`.
    pub fn spot_returns_and_accel(&self, ts_ns: i64) -> SpotAccelStats {
        let r5 = self.trailing_return(ts_ns, 5_000_000_000);
        let r15 = self.trailing_return(ts_ns, 15_000_000_000);
        let r30 = self.trailing_return(ts_ns, 30_000_000_000);

        let accel_15_vs_30 = match (r15, r30) {
            (Some(a), Some(b)) => (a - b) / 15.0,
            _ => 0.0,
        };
        let accel_5_vs_15 = match (r5, r15) {
            (Some(a), Some(b)) => (a - b) / 5.0,
            _ => 0.0,
        };

        SpotAccelStats {
            ret_5s: r5.unwrap_or(0.0),
            ret_15s: r15.unwrap_or(0.0),
            ret_30s: r30.unwrap_or(0.0),
            accel_15s_vs_30s: accel_15_vs_30,
            accel_5s_vs_15s: accel_5_vs_15,
        }
    }
}

/// Aggressor-flow window statistics. See [`SpotHistory::signed_flow_and_adverse`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowStats {
    pub imbalance: f64,
    pub adverse_volume: f64,
    pub large_adverse_count: u32,
    pub intensity: f64,
}

/// Short-horizon spot returns and acceleration proxies.
/// See [`SpotHistory::spot_returns_and_accel`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpotAccelStats {
    pub ret_5s: f64,
    pub ret_15s: f64,
    pub ret_30s: f64,
    pub accel_15s_vs_30s: f64,
    pub accel_5s_vs_15s: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tk(ts_ns: i64, price: f64) -> SpotTick {
        SpotTick {
            ts_ns,
            price,
            quantity: 0.0,
            is_buyer_maker: false,
        }
    }

    #[test]
    fn binary_search_finds_last_price() {
        let h = SpotHistory::new(vec![tk(100, 1.0), tk(200, 2.0), tk(300, 3.0)]);
        assert_eq!(h.price_at_or_before(150), Some(1.0));
        assert_eq!(h.price_at_or_before(200), Some(2.0));
        assert_eq!(h.price_at_or_before(99), None);
        assert_eq!(h.price_at_or_before(400), Some(3.0));
    }

    #[test]
    fn return_computes_over_window() {
        let h = SpotHistory::new(vec![tk(0, 100.0), tk(60_000_000_000, 110.0)]);
        let r = h.simple_return(60_000_000_000, 60_000_000_000).unwrap();
        assert!((r - 0.10).abs() < 1e-9, "got {r}");
    }

    #[test]
    fn range_slices_correctly() {
        let h = SpotHistory::new(vec![tk(100, 1.0), tk(200, 2.0), tk(300, 3.0)]);
        assert_eq!(h.range(150, 250).len(), 1);
        assert_eq!(h.range(0, 1000).len(), 3);
    }

    fn trade(ts_ns: i64, price: f64, quantity: f32, is_buyer_maker: bool) -> SpotTick {
        SpotTick {
            ts_ns,
            price,
            quantity,
            is_buyer_maker,
        }
    }

    #[test]
    fn flow_imbalance_and_adverse_match_reference() {
        let now = 30_000_000_000_i64;
        // Two aggressive buys (is_buyer_maker=false) totalling 30, two aggressive
        // sells (is_buyer_maker=true) totalling 70, one of which is a large print.
        let h = SpotHistory::new(vec![
            trade(now - 25_000_000_000, 100.0, 10.0, false),
            trade(now - 12_000_000_000, 100.0, 20.0, false),
            trade(now - 8_000_000_000, 100.0, 60.0, true),
            trade(now - 2_000_000_000, 100.0, 10.0, true),
        ]);

        // 30s window over a BuyYes load: adverse = sell flow = 70.
        let s = h.signed_flow_and_adverse(now, 30_000_000_000, true);
        assert!((s.imbalance - ((30.0 - 70.0) / 100.0)).abs() < 1e-12);
        assert!((s.adverse_volume - 70.0).abs() < 1e-12);
        assert_eq!(s.large_adverse_count, 1); // only the qty=60 sell clears the 50 threshold
        assert!((s.intensity - (4.0 / 30.0)).abs() < 1e-12);

        // Same window over a BuyNo load: adverse = buy flow = 30, no large buys.
        let s_no = h.signed_flow_and_adverse(now, 30_000_000_000, false);
        assert!((s_no.adverse_volume - 30.0).abs() < 1e-12);
        assert_eq!(s_no.large_adverse_count, 0);

        // 5s window only captures the final sell (qty=10), below the large threshold.
        let s5 = h.signed_flow_and_adverse(now, 5_000_000_000, true);
        assert!((s5.imbalance - (-1.0)).abs() < 1e-12);
        assert!((s5.adverse_volume - 10.0).abs() < 1e-12);
        assert_eq!(s5.large_adverse_count, 0);
        assert!((s5.intensity - (1.0 / 5.0)).abs() < 1e-12);
    }

    #[test]
    fn empty_window_yields_zero_imbalance() {
        let h = SpotHistory::new(vec![trade(0, 100.0, 5.0, false)]);
        let s = h.signed_flow_and_adverse(30_000_000_000, 5_000_000_000, true);
        assert_eq!(s.imbalance, 0.0);
        assert_eq!(s.adverse_volume, 0.0);
        assert_eq!(s.large_adverse_count, 0);
        assert_eq!(s.intensity, 0.0);
    }

    #[test]
    fn spot_returns_and_accel_match_reference() {
        let now = 30_000_000_000_i64;
        // Prices: 100 at -30s, 102 at -15s, 105 at -5s, 106 at now.
        let h = SpotHistory::new(vec![
            trade(now - 30_000_000_000, 100.0, 1.0, false),
            trade(now - 15_000_000_000, 102.0, 1.0, false),
            trade(now - 5_000_000_000, 105.0, 1.0, false),
            trade(now, 106.0, 1.0, false),
        ]);
        let a = h.spot_returns_and_accel(now);
        let r5 = 106.0 / 105.0 - 1.0;
        let r15 = 106.0 / 102.0 - 1.0;
        let r30 = 106.0 / 100.0 - 1.0;
        assert!((a.ret_5s - r5).abs() < 1e-12);
        assert!((a.ret_15s - r15).abs() < 1e-12);
        assert!((a.ret_30s - r30).abs() < 1e-12);
        assert!((a.accel_15s_vs_30s - (r15 - r30) / 15.0).abs() < 1e-12);
        assert!((a.accel_5s_vs_15s - (r5 - r15) / 5.0).abs() < 1e-12);
    }

    #[test]
    fn accel_zero_when_returns_missing() {
        // Only one sample: no price exists before any lookback window opens.
        let h = SpotHistory::new(vec![trade(30_000_000_000, 100.0, 1.0, false)]);
        let a = h.spot_returns_and_accel(30_000_000_000);
        assert_eq!(a.ret_5s, 0.0);
        assert_eq!(a.accel_15s_vs_30s, 0.0);
        assert_eq!(a.accel_5s_vs_15s, 0.0);
    }
}
