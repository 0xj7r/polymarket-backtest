//! Trade tape: each row is an aggressed fill on the Polymarket book.
//! `aggressor_buy = true` means a taker hit the ask (bullish flow);
//! `aggressor_buy = false` means a taker hit the bid (bearish flow).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradeTick {
    pub ts_ns: i64,
    pub price: f32,
    pub size: f32,
    pub aggressor_buy: bool,
}

#[derive(Debug, Default, Clone)]
pub struct TradeHistory {
    samples: Vec<TradeTick>,
}

impl TradeHistory {
    pub fn new(mut samples: Vec<TradeTick>) -> Self {
        samples.sort_by_key(|t| t.ts_ns);
        Self { samples }
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }
    pub fn len(&self) -> usize {
        self.samples.len()
    }
    pub fn samples(&self) -> &[TradeTick] {
        &self.samples
    }

    /// Indices `[start_ns, end_ns]` inclusive.
    pub fn range(&self, start_ns: i64, end_ns: i64) -> &[TradeTick] {
        let lo = self.samples.partition_point(|t| t.ts_ns < start_ns);
        let hi = self.samples.partition_point(|t| t.ts_ns <= end_ns);
        &self.samples[lo..hi]
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TradeFlowSignal {
    /// Buyer-minus-seller volume in the lookback window, normalised to
    /// `[-1, 1]`. Positive = aggressive buying.
    pub flow_imbalance: f32,
    pub buy_volume: f32,
    pub sell_volume: f32,
    pub trade_count: u32,
    /// Mean trade price in the window (price-weighted by size).
    pub vwap: Option<f32>,
}

/// Compute trade-flow imbalance over the trailing `lookback_ns` ending at
/// `now_ns`. Returns `flow = (buy - sell) / (buy + sell)`.
pub fn compute_trade_flow(now_ns: i64, lookback_ns: i64, trades: &TradeHistory) -> TradeFlowSignal {
    let start_ns = now_ns - lookback_ns;
    let slice = trades.range(start_ns, now_ns);
    if slice.is_empty() {
        return TradeFlowSignal::default();
    }
    let mut buy = 0.0f32;
    let mut sell = 0.0f32;
    let mut notional = 0.0f64;
    let mut total_size = 0.0f64;
    for t in slice {
        if t.aggressor_buy {
            buy += t.size;
        } else {
            sell += t.size;
        }
        notional += t.price as f64 * t.size as f64;
        total_size += t.size as f64;
    }
    let total = buy + sell;
    let flow_imbalance = if total > 0.0 {
        ((buy - sell) / total).clamp(-1.0, 1.0)
    } else {
        0.0
    };
    let vwap = if total_size > 0.0 {
        Some((notional / total_size) as f32)
    } else {
        None
    };
    TradeFlowSignal {
        flow_imbalance,
        buy_volume: buy,
        sell_volume: sell,
        trade_count: slice.len() as u32,
        vwap,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tt(ts_ns: i64, size: f32, buy: bool) -> TradeTick {
        TradeTick {
            ts_ns,
            price: 0.5,
            size,
            aggressor_buy: buy,
        }
    }

    #[test]
    fn flow_balances_correctly() {
        let h = TradeHistory::new(vec![
            tt(0, 10.0, true),
            tt(1_000_000_000, 5.0, false),
            tt(2_000_000_000, 7.0, true),
        ]);
        let f = compute_trade_flow(3_000_000_000, 5_000_000_000, &h);
        assert!(f.flow_imbalance > 0.5);
        assert_eq!(f.trade_count, 3);
    }

    #[test]
    fn flow_window_excludes_old() {
        let h = TradeHistory::new(vec![tt(0, 100.0, false), tt(2_000_000_000, 5.0, true)]);
        let f = compute_trade_flow(2_500_000_000, 1_000_000_000, &h);
        // Only the second trade is in window
        assert_eq!(f.trade_count, 1);
        assert!(f.flow_imbalance > 0.99);
    }
}
