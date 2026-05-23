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
            Err(i) if i == 0 => return None,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tk(ts_ns: i64, price: f64) -> SpotTick {
        SpotTick { ts_ns, price, quantity: 0.0, is_buyer_maker: false }
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
}
