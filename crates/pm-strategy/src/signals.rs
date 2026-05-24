//! Signal stack — direction / confidence / calibrated_p / risk.
//!
//! All inputs are `ReplayEvent`s in time order. Functions are deterministic and
//! pure (no globals, no allocation in the hot path). Bigger windows are kept in
//! `Online*` structs that the strategy owns.

use pm_types::ReplayEvent;

/// Rolling fixed-capacity ring buffer of f32. Cheap, allocation-free after init.
#[derive(Debug, Clone)]
pub struct Ring {
    buf: Vec<f32>,
    cap: usize,
    head: usize,
    len: usize,
}

impl Ring {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Self {
            buf: vec![0.0; cap],
            cap,
            head: 0,
            len: 0,
        }
    }
    pub fn push(&mut self, x: f32) {
        self.buf[self.head] = x;
        self.head = (self.head + 1) % self.cap;
        if self.len < self.cap {
            self.len += 1;
        }
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_full(&self) -> bool {
        self.len == self.cap
    }
    pub fn oldest(&self) -> Option<f32> {
        if self.len == 0 {
            return None;
        }
        let idx = if self.len < self.cap { 0 } else { self.head };
        Some(self.buf[idx])
    }
    pub fn newest(&self) -> Option<f32> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + self.cap - 1) % self.cap;
        Some(self.buf[idx])
    }
    pub fn mean(&self) -> f32 {
        if self.len == 0 {
            return 0.0;
        }
        let sum: f32 = (0..self.len)
            .map(|i| self.buf[(self.head + self.cap - self.len + i) % self.cap])
            .sum();
        sum / self.len as f32
    }
    pub fn std(&self) -> f32 {
        if self.len < 2 {
            return 0.0;
        }
        let m = self.mean();
        let var: f32 = (0..self.len)
            .map(|i| {
                let v = self.buf[(self.head + self.cap - self.len + i) % self.cap];
                (v - m).powi(2)
            })
            .sum::<f32>()
            / (self.len - 1) as f32;
        var.sqrt()
    }
    /// Count sign flips (consecutive samples with opposite signs).
    pub fn sign_flips(&self) -> usize {
        if self.len < 2 {
            return 0;
        }
        let mut prev = 0.0f32;
        let mut have_prev = false;
        let mut flips = 0;
        for i in 0..self.len {
            let v = self.buf[(self.head + self.cap - self.len + i) % self.cap];
            if v == 0.0 {
                continue;
            }
            if have_prev && (v.signum() != prev.signum()) {
                flips += 1;
            }
            prev = v;
            have_prev = true;
        }
        flips
    }
}

/// Microprice: size-weighted midpoint of best bid/ask. Pulls the implied
/// price toward whichever side is larger (the side with less queue depth
/// to clear).
///
///   microprice = (bid * ask_size + ask * bid_size) / (bid_size + ask_size)
pub fn microprice(yes_bid: f32, yes_ask: f32, bid_size: f32, ask_size: f32) -> f32 {
    if bid_size + ask_size <= 0.0 || yes_bid <= 0.0 || yes_ask <= 0.0 {
        return 0.5 * (yes_bid + yes_ask);
    }
    (yes_bid * ask_size + yes_ask * bid_size) / (bid_size + ask_size)
}

/// Order-flow imbalance: instantaneous bid/ask size imbalance, clipped to
/// `[-1, 1]`. Positive = more on bid (buying pressure).
pub fn ofi(bid_size: f32, ask_size: f32) -> f32 {
    let s = bid_size + ask_size;
    if s <= 0.0 {
        return 0.0;
    }
    let raw = (bid_size - ask_size) / s;
    raw.clamp(-1.0, 1.0)
}

/// Liquidity: average top-N depth (yes side), in shares. Strategies use it as
/// a fill-confidence proxy.
pub fn top_n_depth(event: &ReplayEvent, n: usize) -> f32 {
    let n = n.min(event.bids.len());
    let bid_d: f32 = event.bids[..n].iter().map(|l| l.size).sum();
    let ask_d: f32 = event.asks[..n].iter().map(|l| l.size).sum();
    0.5 * (bid_d + ask_d)
}

/// Aggregated direction signal. Positive => price expected to drift up (YES).
#[derive(Debug, Clone, Copy, Default)]
pub struct DirectionScore {
    pub momentum: f32,       // recent mid drift, normalised to [-1, 1]
    pub ofi: f32,            // [-1, 1]
    pub microprice_dev: f32, // (microprice - mid) / spread, [-1, 1]
    pub composite: f32,      // weighted combination, [-1, 1]
}

pub fn direction_score(
    event: &ReplayEvent,
    recent_mids: &Ring,
    micro_dev_scale: f32,
) -> DirectionScore {
    let momentum = if let (Some(old), Some(new)) = (recent_mids.oldest(), recent_mids.newest()) {
        // 5x scaling: a 10c drift over the window pegs at +/-0.5.
        ((new - old) * 5.0).clamp(-1.0, 1.0)
    } else {
        0.0
    };

    let ofi_v = ofi(event.bids[0].size, event.asks[0].size);

    let mid = event.yes_mid;
    let mp = microprice(
        event.yes_bid,
        event.yes_ask,
        event.bids[0].size,
        event.asks[0].size,
    );
    let spread = (event.yes_ask - event.yes_bid).max(1e-6);
    let microprice_dev = (((mp - mid) / spread) * micro_dev_scale).clamp(-1.0, 1.0);

    // Weights are deliberately simple — tune as evidence justifies.
    let composite = (0.5 * momentum + 0.3 * ofi_v + 0.2 * microprice_dev).clamp(-1.0, 1.0);

    DirectionScore {
        momentum,
        ofi: ofi_v,
        microprice_dev,
        composite,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConfidenceScore {
    pub stability: f32,            // [0, 1] — 1 = perfectly consistent recent signal
    pub early_market_penalty: f32, // [0, 1] — 1 = fully damped (very early)
    pub composite: f32,            // [0, 1]
}

/// `recent_dir_scores` is the sliding window of recent composite direction
/// scores. Stability is `1 - clamp(std / 0.5, 0, 1)`.
///
/// `seconds_since_window_open` should be measured from the strategy's chosen
/// betting-window open (e.g., resolution - 5m), not the tape start.
pub fn confidence_score(
    recent_dir_scores: &Ring,
    seconds_since_window_open: f32,
) -> ConfidenceScore {
    let std_v = recent_dir_scores.std();
    let stability = (1.0 - (std_v / 0.5).clamp(0.0, 1.0)).clamp(0.0, 1.0);

    // Linear ramp: 1.0 at t=0, 0.0 at t=60s, then 0 thereafter.
    let early_market_penalty = if seconds_since_window_open >= 60.0 {
        0.0
    } else if seconds_since_window_open <= 0.0 {
        1.0
    } else {
        1.0 - (seconds_since_window_open / 60.0)
    };

    let composite = (stability * (1.0 - early_market_penalty)).clamp(0.0, 1.0);

    ConfidenceScore {
        stability,
        early_market_penalty,
        composite,
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RiskScore {
    pub whipsaw: f32,   // [0, 1] — 1 = lots of direction sign flips
    pub liquidity: f32, // [0, 1] — 1 = thick book
    pub path_risk: f32, // [0, 1] — 1 = mid pinned near 0.5 (high path risk)
    pub composite: f32, // [0, 1] — higher = MORE risk
}

pub fn risk_score(
    event: &ReplayEvent,
    recent_dir_scores: &Ring,
    depth_full_at_shares: f32,
) -> RiskScore {
    let flips = recent_dir_scores.sign_flips() as f32;
    let denom = (recent_dir_scores.len().saturating_sub(1).max(1)) as f32;
    let whipsaw = (flips / denom).clamp(0.0, 1.0);

    let depth = top_n_depth(event, 5);
    let liquidity = (depth / depth_full_at_shares).clamp(0.0, 1.0);

    // 1.0 at mid=0.5, 0.0 at mid in {0, 1}. Triangular function.
    let path_risk = 1.0 - (2.0 * (event.yes_mid - 0.5).abs()).clamp(0.0, 1.0);

    // Composite: whipsaw + path_risk are bads; liquidity scales them down.
    let bads = 0.6 * whipsaw + 0.4 * path_risk;
    let composite = (bads * (1.0 - 0.5 * liquidity)).clamp(0.0, 1.0);

    RiskScore {
        whipsaw,
        liquidity,
        path_risk,
        composite,
    }
}

/// Blend (direction, confidence) into a calibrated YES-win probability,
/// hard-bounded to `[0.06, 0.94]` (symmetric around 0.5).
///
/// The previous version clamped to `[0.55, 0.94]` which structurally forced
/// the strategy long YES even on bearish signals — a directional bias that
/// blew up on Down-resolving markets.
///
/// `book_implied` is the market's own midpoint (book-implied probability).
/// `model_weight` selects how much the model overrides the book.
pub fn calibrated_p(direction: f32, confidence: f32, book_implied: f32, model_weight: f32) -> f32 {
    let edge = (direction * confidence * 0.4).clamp(-0.4, 0.4);
    let model = (0.5 + edge).clamp(0.0, 1.0);
    let blended = model_weight * model + (1.0 - model_weight) * book_implied;
    blended.clamp(0.06, 0.94)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evt(bid: f32, ask: f32, bid_size: f32, ask_size: f32) -> ReplayEvent {
        use pm_types::{BookLevel, MarketId, ReplayFlags, tape::TAPE_DEPTH};
        let mut bids = [BookLevel::default(); TAPE_DEPTH];
        let mut asks = [BookLevel::default(); TAPE_DEPTH];
        bids[0] = BookLevel {
            price: bid,
            size: bid_size,
        };
        asks[0] = BookLevel {
            price: ask,
            size: ask_size,
        };
        ReplayEvent {
            ts_ns: 0,
            market_id: MarketId(1),
            yes_mid: 0.5 * (bid + ask),
            yes_bid: bid,
            yes_ask: ask,
            volume: 0.0,
            bids,
            asks,
            spot_price: 0.0,
            flags: ReplayFlags::BOOK_UPDATE,
        }
    }

    #[test]
    fn microprice_centres_on_mid_when_balanced() {
        let mp = microprice(0.50, 0.51, 100.0, 100.0);
        assert!((mp - 0.505).abs() < 1e-6, "got {mp}");
    }

    #[test]
    fn microprice_pulls_toward_thin_side() {
        // Thin ask side: microprice should sit above the mid (toward the ask).
        let mp = microprice(0.50, 0.51, 100.0, 10.0);
        assert!(mp > 0.505, "expected > 0.505, got {mp}");
    }

    #[test]
    fn ofi_signs_correctly() {
        assert!((ofi(100.0, 100.0)).abs() < 1e-6);
        assert!(ofi(200.0, 50.0) > 0.0);
        assert!(ofi(50.0, 200.0) < 0.0);
    }

    #[test]
    fn ring_mean_std_basic() {
        let mut r = Ring::new(3);
        r.push(1.0);
        r.push(2.0);
        r.push(3.0);
        assert!((r.mean() - 2.0).abs() < 1e-6);
        assert!(r.std() > 0.0);
    }

    #[test]
    fn ring_sign_flips_counts() {
        let mut r = Ring::new(6);
        r.push(0.5);
        r.push(-0.4);
        r.push(0.3);
        r.push(-0.2);
        r.push(0.1);
        assert_eq!(r.sign_flips(), 4);
    }

    #[test]
    fn confidence_early_market_penalty_decays() {
        let mut r = Ring::new(8);
        for _ in 0..8 {
            r.push(0.2);
        }
        let c0 = confidence_score(&r, 0.0);
        let c30 = confidence_score(&r, 30.0);
        let c60 = confidence_score(&r, 60.0);
        assert!(c0.early_market_penalty > 0.99);
        assert!(c30.early_market_penalty > 0.49 && c30.early_market_penalty < 0.51);
        assert!(c60.early_market_penalty < 0.01);
        assert!(c60.composite > c0.composite);
    }

    #[test]
    fn calibrated_p_bounded() {
        // Extreme bullish: should saturate at upper bound 0.94.
        let p = calibrated_p(1.0, 1.0, 0.99, 1.0);
        assert!(p <= 0.94 + 1e-6, "got {p}");
        // Extreme bearish: should saturate at LOWER bound (not 0.55 — that
        // bug forced the strategy long-YES always).
        let p_low = calibrated_p(-1.0, 1.0, 0.01, 1.0);
        assert!(p_low <= 0.10 + 1e-6, "got {p_low} (expected <= 0.10)");
        assert!(p_low >= 0.06 - 1e-6);
    }

    #[test]
    fn risk_pinned_near_mid_is_high_path_risk() {
        let e_pinned = evt(0.50, 0.51, 100.0, 100.0);
        let e_polarised = evt(0.05, 0.06, 100.0, 100.0);
        let mut r = Ring::new(5);
        r.push(0.1);
        r.push(0.1);
        let s_pinned = risk_score(&e_pinned, &r, 1000.0);
        let s_polarised = risk_score(&e_polarised, &r, 1000.0);
        assert!(
            s_pinned.path_risk > s_polarised.path_risk,
            "pinned {} polar {}",
            s_pinned.path_risk,
            s_polarised.path_risk
        );
    }
}
