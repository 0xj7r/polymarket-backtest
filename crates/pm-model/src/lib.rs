//! Model core for phase 2 rollout: reusable 4-score output block.
//!
//! This module intentionally has no policy/risk execution logic so strategies can
//! consume it across backtest/paper/live implementations.

#![forbid(unsafe_code)]

use pm_types::{ReplayEvent, SpotHistory};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const NS_PER_SECOND: i64 = 1_000_000_000;
const MOMENTUM_WINDOWS_SECONDS: [i64; 5] = [10, 30, 60, 120, 300];
const MOMENTUM_WEIGHTS: [f32; 5] = [0.45, 0.25, 0.15, 0.10, 0.05];
const SKEW_BINS: usize = 10;
const HOUR_BINS: usize = 24;
const META_FEATURES: usize = 75;
const META_CALIBRATOR_LR: f32 = 0.08;
const META_CALIBRATOR_MIN_UPDATES: u32 = 12;
const META_CALIBRATOR_WEIGHT_DECAY: f32 = 1.0e-3;
const META_CALIBRATOR_WEIGHT_CLIP: f32 = 1.0;
const META_TREE_COUNT: usize = 10;
const META_TREE_LEARNING_RATE: f32 = 0.08;
const META_TREE_L2: f32 = 8.0;
const META_TREE_MIN_LEAF: usize = 128;
const META_TREE_MAX_TRAIN_SAMPLES: usize = 48_000;
const META_TREE_VALUE_CLIP: f32 = 0.65;
const VOLATILITY_WINDOW_MARKETS: usize = 3;
const VOLATILITY_REGIME_WEIGHT: f32 = 0.22;
const TIME_OF_DAY_REGIME_WEIGHT: f32 = 0.18;
const BTC_WHIPSAW_RISK_WEIGHT: f32 = 0.16;
const BTC_PATH_INEFFICIENCY_RISK_WEIGHT: f32 = 0.10;
const BTC_REVERSAL_PRESSURE_RISK_WEIGHT: f32 = 0.12;
const BETA_CALIBRATOR_MIN_SAMPLES: usize = 256;
const BETA_CALIBRATOR_EPOCHS: usize = 96;
const BETA_CALIBRATOR_LR: f32 = 0.03;
const BETA_CALIBRATOR_L2: f32 = 0.01;
const BETA_CALIBRATOR_COEFF_CLIP: f32 = 5.0;
const ISOTONIC_MIN_SAMPLES: usize = 256;
const ISOTONIC_SHRINKAGE: f32 = 500.0;
const OBSERVED_RANGE_HIGH_CERT_RISK_WEIGHT: f32 = 0.35;

pub const META_FEATURE_NAMES: [&str; META_FEATURES] = [
    "direction_score_side",
    "momentum_side",
    "momentum_10s_side",
    "momentum_30s_side",
    "momentum_60s_side",
    "momentum_120s_side",
    "momentum_300s_side",
    "momentum_10_60_accel_side",
    "momentum_30_300_accel_side",
    "spot_momentum_side",
    "ofi_side",
    "microprice_dev_side",
    "microprice_spot_alignment_side",
    "top3_delta_5s_side",
    "top3_delta_15s_side",
    "direction_composite_side",
    "book_imbalance_side",
    "stability",
    "sign_persistence",
    "markov_persistence",
    "early_market_penalty",
    "confidence_composite",
    "whipsaw",
    "liquidity",
    "path_risk",
    "imbalance_turn",
    "markov_reversal_risk",
    "skew_penalty",
    "risk_composite",
    "market_mid",
    "volatility_penalty",
    "time_of_day_penalty",
    "time_of_day_advantage",
    "volatility_regime",
    "dir_mean_3_side",
    "dir_mean_8_side",
    "dir_slope_8_side",
    "abs_direction_score",
    "mid_distance_from_half",
    "confidence_liquidity_interaction",
    "spot_fast_momentum_side",
    "spot_broad_momentum_side",
    "spot_fast_broad_alignment",
    "spot_acceleration_side",
    "spot_momentum_600s_side",
    "spot_momentum_900s_side",
    "spot_momentum_1800s_side",
    "spot_momentum_3600s_side",
    "spot_fast_long_alignment",
    "spot_broad_trend_consistency",
    "spot_broad_acceleration_side",
    "dir_flip_rate_8",
    "dir_std_8",
    "dir_abs_mean_8",
    "side_market_price",
    "pre_meta_edge_vs_side_price",
    "seconds_in_window_norm",
    "final_120s_norm",
    "high_cert_side_price",
    "side_favourite_skew",
    "observed_yes_range_so_far",
    "observed_range_high_cert_interaction",
    "btc_whipsaw_score",
    "btc_path_efficiency",
    "btc_path_inefficiency",
    "btc_sign_flip_rate",
    "btc_realized_vol_180s",
    "btc_reversal_pressure",
    "side_high_cert_path_inefficiency",
    "side_high_cert_reversal_pressure",
    "spot_momentum_7200s_side",
    "spot_momentum_14400s_side",
    "spot_1h_4h_alignment",
    "spot_ultra_trend_consistency",
    "spot_ultra_acceleration_side",
];

/// Output contract expected by the 4-score engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModelOutput {
    /// -1.0 to +1.0 directional score.
    pub direction_score: f32,
    /// 0.0 to 1.0 confidence score.
    pub confidence_score: f32,
    /// Calibrated probability for the predicted side in `[0.55, 0.94]`.
    pub calibrated_p: f32,
    /// 0.0 to 1.0 risk score (higher means riskier).
    pub risk_score: f32,
}

impl Default for ModelOutput {
    fn default() -> Self {
        Self {
            direction_score: 0.0,
            confidence_score: 0.0,
            calibrated_p: 0.55,
            risk_score: 1.0,
        }
    }
}

/// Feature-level attribution for one model evaluation.
#[derive(Debug, Clone, Copy)]
pub struct ModelAttribution {
    pub meta_features: MetaFeatures,
    pub direction: DirectionScore,
    pub confidence: ConfidenceScore,
    pub risk: RiskScore,
    pub sequence: SequenceFeatures,
    pub book_imbalance_top3: f32,
    pub spot_score: f32,
    pub direction_raw: f32,
    pub direction_side_is_yes: bool,
    pub side_probability_pre_meta: f32,
    pub side_probability_post_meta: f32,
    pub volatility_regime: f32,
    pub time_of_day_edge: f32,
    pub observed_yes_range_so_far: f32,
    pub observed_range_high_cert_interaction: f32,
    pub spot_regime: SpotRegimeFeatures,
    pub meta_calibrator_updates: u32,
}

impl Default for ModelAttribution {
    fn default() -> Self {
        Self {
            meta_features: MetaFeatures::default(),
            direction: DirectionScore::default(),
            confidence: ConfidenceScore::default(),
            risk: RiskScore::default(),
            sequence: SequenceFeatures::default(),
            book_imbalance_top3: 0.0,
            spot_score: 0.0,
            direction_raw: 0.0,
            direction_side_is_yes: true,
            side_probability_pre_meta: 0.5,
            side_probability_post_meta: 0.5,
            volatility_regime: 0.0,
            time_of_day_edge: 0.5,
            observed_yes_range_so_far: 0.0,
            observed_range_high_cert_interaction: 0.0,
            spot_regime: SpotRegimeFeatures::default(),
            meta_calibrator_updates: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ModelEvaluation {
    pub output: ModelOutput,
    pub attribution: ModelAttribution,
}

/// Rolling fixed-capacity ring buffer.
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
    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
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
    pub fn mean_last_n(&self, n: usize) -> f32 {
        let n = n.min(self.len);
        if n == 0 {
            return 0.0;
        }
        let start = self.len - n;
        let sum: f32 = (start..self.len)
            .map(|i| self.buf[(self.head + self.cap - self.len + i) % self.cap])
            .sum();
        sum / n as f32
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
    pub fn std_last_n(&self, n: usize) -> f32 {
        let n = n.min(self.len);
        if n < 2 {
            return 0.0;
        }
        let start = self.len - n;
        let mean = self.mean_last_n(n);
        let var: f32 = (start..self.len)
            .map(|i| {
                let v = self.buf[(self.head + self.cap - self.len + i) % self.cap];
                (v - mean).powi(2)
            })
            .sum::<f32>()
            / (n - 1) as f32;
        var.sqrt()
    }
    /// Count sign flips (consecutive samples with opposite signs).
    pub fn sign_flips(&self) -> usize {
        self.sign_flips_last_n(self.len)
    }

    /// Count sign flips over the last `n` samples (minimum 2 required).
    pub fn sign_flips_last_n(&self, n: usize) -> usize {
        if self.len < 2 {
            return 0;
        }
        let n = n.min(self.len);
        if n < 2 {
            return 0;
        }
        let start = self.len - n;
        let mut prev = 0.0f32;
        let mut have_prev = false;
        let mut flips = 0;
        for i in start..self.len {
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

#[derive(Debug, Clone)]
pub struct TimedRing {
    ts_ns: Vec<i64>,
    values: Vec<f32>,
    cap: usize,
    head: usize,
    len: usize,
}

impl TimedRing {
    pub fn new(cap: usize) -> Self {
        assert!(cap > 0);
        Self {
            ts_ns: vec![0; cap],
            values: vec![0.0; cap],
            cap,
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, ts_ns: i64, value: f32) {
        self.ts_ns[self.head] = ts_ns;
        self.values[self.head] = value;
        self.head = (self.head + 1) % self.cap;
        if self.len < self.cap {
            self.len += 1;
        }
    }
    pub fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn newest(&self) -> Option<(i64, f32)> {
        if self.len == 0 {
            return None;
        }
        let idx = (self.head + self.cap - 1) % self.cap;
        Some((self.ts_ns[idx], self.values[idx]))
    }

    pub fn value_at_or_before(&self, target_ts_ns: i64) -> Option<(i64, f32)> {
        if self.len == 0 {
            return None;
        }
        for i in (0..self.len).rev() {
            let idx = (self.head + self.cap - 1 - i) % self.cap;
            if self.ts_ns[idx] <= target_ts_ns {
                return Some((self.ts_ns[idx], self.values[idx]));
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct DirectionScore {
    pub momentum: f32,
    pub momentum_10s: f32,
    pub momentum_30s: f32,
    pub momentum_60s: f32,
    pub momentum_120s: f32,
    pub momentum_300s: f32,
    pub momentum_10_60_accel: f32,
    pub momentum_30_300_accel: f32,
    pub spot_momentum: f32,
    pub spot_fast_momentum: f32,
    pub spot_broad_momentum: f32,
    pub spot_momentum_600s: f32,
    pub spot_momentum_900s: f32,
    pub spot_momentum_1800s: f32,
    pub spot_momentum_3600s: f32,
    pub spot_momentum_7200s: f32,
    pub spot_momentum_14400s: f32,
    pub spot_1h_4h_alignment: f32,
    pub spot_ultra_trend_consistency: f32,
    pub spot_ultra_acceleration: f32,
    pub spot_fast_broad_alignment: f32,
    pub spot_fast_long_alignment: f32,
    pub spot_broad_trend_consistency: f32,
    pub spot_acceleration: f32,
    pub spot_broad_acceleration: f32,
    pub ofi: f32,
    pub microprice_dev: f32,
    pub microprice_spot_alignment: f32,
    pub top3_delta_5s: f32,
    pub top3_delta_15s: f32,
    pub composite: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ConfidenceScore {
    pub stability: f32,
    pub sign_persistence: f32,
    pub markov_persistence: f32,
    pub early_market_penalty: f32,
    pub time_of_day_advantage: f32,
    pub composite: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RiskScore {
    pub whipsaw: f32,
    pub liquidity: f32,
    pub path_risk: f32,
    pub imbalance_turn: f32,
    pub markov_reversal_risk: f32,
    pub skew_penalty: f32,
    pub observed_range_high_cert_risk: f32,
    pub btc_whipsaw_risk: f32,
    pub btc_path_inefficiency_risk: f32,
    pub btc_reversal_pressure: f32,
    pub volatility_penalty: f32,
    pub time_of_day_penalty: f32,
    pub composite: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SpotRegimeFeatures {
    pub whipsaw_score: f32,
    pub path_efficiency: f32,
    pub path_inefficiency: f32,
    pub sign_flip_rate: f32,
    pub realized_vol_180s_bps: f32,
    pub reversal_pressure: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SequenceFeatures {
    pub dir_mean_3: f32,
    pub dir_mean_8: f32,
    pub dir_slope_8: f32,
    pub dir_flip_rate_8: f32,
    pub dir_std_8: f32,
    pub dir_abs_mean_8: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct MarketHistoryPoint {
    pub min_mid: f32,
    pub max_mid: f32,
    pub final_mid: f32,
}

#[derive(Debug, Clone)]
struct VolatilityState {
    current_market_id: Option<u32>,
    current_min_mid: f32,
    current_max_mid: f32,
    current_started: bool,
    prev_market_close: Option<f32>,
    recent_ranges: Ring,
}

#[derive(Debug, Clone, Copy, Default)]
struct DirectionMarkov {
    last_side: Option<bool>,
    transitions: [[u32; 2]; 2],
    observations: u32,
}

impl DirectionMarkov {
    fn observe(&mut self, side: bool) -> f32 {
        if let Some(prev) = self.last_side {
            let from = usize::from(prev);
            let to = usize::from(side);
            self.transitions[from][to] = self.transitions[from][to].saturating_add(1);
            self.observations = self.observations.saturating_add(1);
            let row = self.transitions[from];
            let total = (row[0] + row[1]) as f32;
            self.last_side = Some(side);
            if total <= 0.0 {
                0.5
            } else {
                (row[from] as f32 / total).clamp(0.0, 1.0)
            }
        } else {
            self.last_side = Some(side);
            self.observations = self.observations.saturating_add(1);
            0.5
        }
    }
}

impl Default for VolatilityState {
    fn default() -> Self {
        Self {
            current_market_id: None,
            current_min_mid: 0.0,
            current_max_mid: 0.0,
            current_started: false,
            prev_market_close: None,
            recent_ranges: Ring::new(VOLATILITY_WINDOW_MARKETS),
        }
    }
}

impl VolatilityState {
    fn on_event(&mut self, event: &ReplayEvent) {
        let mid = event.yes_mid;
        if self.current_market_id != Some(event.market_id.0) || !self.current_started {
            self.current_market_id = Some(event.market_id.0);
            self.current_min_mid = mid;
            self.current_max_mid = mid;
            self.current_started = true;
            return;
        }
        self.current_min_mid = self.current_min_mid.min(mid);
        self.current_max_mid = self.current_max_mid.max(mid);
    }

    fn finalize_market(&mut self, close_mid: f32) -> Option<MarketHistoryPoint> {
        if !self.current_started {
            return None;
        }
        let point = MarketHistoryPoint {
            min_mid: self.current_min_mid,
            max_mid: self.current_max_mid,
            final_mid: close_mid,
        };
        self.current_market_id = None;
        self.current_started = false;
        let range = (point.max_mid - point.min_mid).abs();
        let true_range = if let Some(prev) = self.prev_market_close {
            (range).max((close_mid - prev).abs())
        } else {
            range
        };
        self.recent_ranges.push(true_range);
        self.prev_market_close = Some(close_mid);
        Some(point)
    }

    fn atr_like(&self) -> f32 {
        if self.recent_ranges.len() == 0 {
            return 0.0;
        }
        self.recent_ranges.mean()
    }
}

#[derive(Debug, Clone, Copy)]
struct TimeOfDayStats {
    pub taken: u32,
    pub yes_wins: u32,
}

#[derive(Debug, Clone)]
struct TimeOfDayTable {
    buckets: [TimeOfDayStats; HOUR_BINS],
}

impl Default for TimeOfDayTable {
    fn default() -> Self {
        Self {
            buckets: [TimeOfDayStats {
                taken: 0,
                yes_wins: 0,
            }; HOUR_BINS],
        }
    }
}

impl TimeOfDayTable {
    fn record(&mut self, hour_bucket: usize, predicted_yes: bool, resolved_yes: bool) {
        let idx = hour_bucket % HOUR_BINS;
        let b = &mut self.buckets[idx];
        b.taken = b.taken.saturating_add(1);
        if predicted_yes && resolved_yes || !predicted_yes && !resolved_yes {
            b.yes_wins = b.yes_wins.saturating_add(1);
        }
    }

    fn expected_side_win_rate(&self, hour_bucket: usize, predicted_yes: bool) -> f32 {
        let b = self.buckets[hour_bucket % HOUR_BINS];
        if b.taken == 0 {
            return 0.5;
        }
        if predicted_yes {
            (b.yes_wins as f32 / b.taken as f32).clamp(0.0, 1.0)
        } else if b.taken == 0 {
            0.5
        } else {
            (1.0 - (b.yes_wins as f32 / b.taken as f32)).clamp(0.0, 1.0)
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetaFeatures {
    #[serde(
        serialize_with = "serialize_f32_array",
        deserialize_with = "deserialize_f32_array"
    )]
    pub values: [f32; META_FEATURES],
}

impl Default for MetaFeatures {
    fn default() -> Self {
        Self {
            values: [0.0; META_FEATURES],
        }
    }
}

impl MetaFeatures {
    pub fn from_raw(
        direction_score: f32,
        direction: DirectionScore,
        confidence: ConfidenceScore,
        risk: RiskScore,
        sequence: SequenceFeatures,
        volatility_regime: f32,
        book_imbalance: f32,
        market_mid: f32,
        side_probability_pre_meta: f32,
        observed_yes_range_so_far: f32,
        spot_regime: SpotRegimeFeatures,
        seconds_since_window_open: f32,
    ) -> Self {
        let side = if direction_score >= 0.0 { 1.0 } else { -1.0 };
        let side_market_price = if side > 0.0 {
            market_mid
        } else {
            1.0 - market_mid
        }
        .clamp(0.0, 1.0);
        let mut values = [0.0; META_FEATURES];
        let mut idx = 0;
        // Direction raw + composite.
        values[idx] = (direction_score * side).clamp(-1.0, 1.0);
        idx += 1;
        values[idx] = direction.momentum * side;
        idx += 1;
        values[idx] = direction.momentum_10s * side;
        idx += 1;
        values[idx] = direction.momentum_30s * side;
        idx += 1;
        values[idx] = direction.momentum_60s * side;
        idx += 1;
        values[idx] = direction.momentum_120s * side;
        idx += 1;
        values[idx] = direction.momentum_300s * side;
        idx += 1;
        values[idx] = direction.momentum_10_60_accel * side;
        idx += 1;
        values[idx] = direction.momentum_30_300_accel * side;
        idx += 1;
        values[idx] = direction.spot_momentum * side;
        idx += 1;
        values[idx] = direction.ofi * side;
        idx += 1;
        values[idx] = direction.microprice_dev * side;
        idx += 1;
        values[idx] = direction.microprice_spot_alignment * side;
        idx += 1;
        values[idx] = direction.top3_delta_5s * side;
        idx += 1;
        values[idx] = direction.top3_delta_15s * side;
        idx += 1;
        values[idx] = direction.composite * side;
        idx += 1;
        values[idx] = book_imbalance.clamp(-1.0, 1.0) * side;
        idx += 1;

        // Confidence stack.
        values[idx] = confidence.stability;
        idx += 1;
        values[idx] = confidence.sign_persistence;
        idx += 1;
        values[idx] = confidence.markov_persistence;
        idx += 1;
        values[idx] = confidence.early_market_penalty;
        idx += 1;
        values[idx] = confidence.composite;
        idx += 1;

        // Risk stack.
        values[idx] = risk.whipsaw;
        idx += 1;
        values[idx] = risk.liquidity;
        idx += 1;
        values[idx] = risk.path_risk;
        idx += 1;
        values[idx] = risk.imbalance_turn;
        idx += 1;
        values[idx] = risk.markov_reversal_risk;
        idx += 1;
        values[idx] = risk.skew_penalty;
        idx += 1;
        values[idx] = risk.composite;
        idx += 1;
        values[idx] = market_mid.clamp(0.0, 1.0);
        idx += 1;
        values[idx] = risk.volatility_penalty;
        idx += 1;
        values[idx] = risk.time_of_day_penalty;
        idx += 1;
        values[idx] = confidence.time_of_day_advantage.clamp(-1.0, 1.0);
        idx += 1;
        values[idx] = volatility_regime.clamp(0.0, 1.0);
        idx += 1;
        values[idx] = sequence.dir_mean_3 * side;
        idx += 1;
        values[idx] = sequence.dir_mean_8 * side;
        idx += 1;
        values[idx] = sequence.dir_slope_8 * side;
        idx += 1;
        values[idx] = direction_score.abs().clamp(0.0, 1.0);
        idx += 1;
        values[idx] = (2.0 * (market_mid - 0.5).abs()).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = (confidence.composite * risk.liquidity).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = direction.spot_fast_momentum * side;
        idx += 1;
        values[idx] = direction.spot_broad_momentum * side;
        idx += 1;
        values[idx] = direction.spot_fast_broad_alignment;
        idx += 1;
        values[idx] = direction.spot_acceleration * side;
        idx += 1;
        values[idx] = direction.spot_momentum_600s * side;
        idx += 1;
        values[idx] = direction.spot_momentum_900s * side;
        idx += 1;
        values[idx] = direction.spot_momentum_1800s * side;
        idx += 1;
        values[idx] = direction.spot_momentum_3600s * side;
        idx += 1;
        values[idx] = direction.spot_fast_long_alignment;
        idx += 1;
        values[idx] = direction.spot_broad_trend_consistency;
        idx += 1;
        values[idx] = direction.spot_broad_acceleration * side;
        idx += 1;
        values[idx] = sequence.dir_flip_rate_8;
        idx += 1;
        values[idx] = sequence.dir_std_8;
        idx += 1;
        values[idx] = sequence.dir_abs_mean_8;
        idx += 1;
        values[idx] = side_market_price;
        idx += 1;
        values[idx] = (side_probability_pre_meta - side_market_price).clamp(-1.0, 1.0);
        idx += 1;
        values[idx] = (seconds_since_window_open / 300.0).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = ((seconds_since_window_open - 180.0) / 120.0).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = ((side_market_price - 0.85) / 0.15).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = (2.0 * (side_market_price - 0.5)).clamp(-1.0, 1.0);
        idx += 1;
        let observed_range = observed_yes_range_so_far.clamp(0.0, 1.0);
        values[idx] = observed_range;
        idx += 1;
        values[idx] =
            (observed_range * ((side_market_price - 0.75) / 0.25).clamp(0.0, 1.0)).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = spot_regime.whipsaw_score.clamp(0.0, 1.0);
        idx += 1;
        values[idx] = spot_regime.path_efficiency.clamp(0.0, 1.0);
        idx += 1;
        values[idx] = spot_regime.path_inefficiency.clamp(0.0, 1.0);
        idx += 1;
        values[idx] = spot_regime.sign_flip_rate.clamp(0.0, 1.0);
        idx += 1;
        values[idx] = (spot_regime.realized_vol_180s_bps / 12.0).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = spot_regime.reversal_pressure.clamp(0.0, 1.0);
        idx += 1;
        let high_cert = ((side_market_price - 0.82) / 0.15).clamp(0.0, 1.0);
        values[idx] = (high_cert * spot_regime.path_inefficiency).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = (high_cert * spot_regime.reversal_pressure).clamp(0.0, 1.0);
        idx += 1;
        values[idx] = direction.spot_momentum_7200s * side;
        idx += 1;
        values[idx] = direction.spot_momentum_14400s * side;
        idx += 1;
        values[idx] = direction.spot_1h_4h_alignment;
        idx += 1;
        values[idx] = direction.spot_ultra_trend_consistency;
        idx += 1;
        values[idx] = direction.spot_ultra_acceleration * side;
        idx += 1;
        debug_assert_eq!(idx, META_FEATURES);
        Self { values }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct MetaTrainingSample {
    pub features: MetaFeatures,
    #[serde(default)]
    pub market_idx: u32,
    /// Predicted-side probability before the meta-calibrator adjustment.
    pub base_side_probability: f32,
    /// True if the predicted side won at resolution.
    pub side_observed: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
pub struct MetaTrainingConfig {
    pub epochs: usize,
    pub learning_rate: f32,
    pub l2: f32,
    pub weight_clip: f32,
    pub reset_before_fit: bool,
}

impl Default for MetaTrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 24,
            learning_rate: 0.04,
            l2: 1.0e-3,
            weight_clip: META_CALIBRATOR_WEIGHT_CLIP,
            reset_before_fit: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct MetaTrainingStats {
    pub samples: usize,
    pub epochs: usize,
    pub updates: u32,
    pub log_loss: f32,
}

#[derive(Debug, Clone, Copy)]
struct PendingMetaTrainingSample {
    features: MetaFeatures,
    base_side_probability: f32,
}

#[derive(Debug, Clone)]
pub struct OnlineMetaCalibrator {
    weights: [f32; META_FEATURES],
    bias: f32,
    beta: BetaCalibrator,
    isotonic: IsotonicCalibrator,
    trees: TreeEnsemble,
    updates: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OnlineMetaCalibratorSnapshot {
    #[serde(
        serialize_with = "serialize_f32_array",
        deserialize_with = "deserialize_f32_array"
    )]
    pub weights: [f32; META_FEATURES],
    pub bias: f32,
    #[serde(default)]
    pub beta: BetaCalibratorSnapshot,
    #[serde(default)]
    pub isotonic: IsotonicCalibratorSnapshot,
    #[serde(default)]
    pub trees: TreeEnsembleSnapshot,
    pub updates: u32,
}

fn serialize_f32_array<S, const N: usize>(
    values: &[f32; N],
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    values.as_slice().serialize(serializer)
}

fn deserialize_f32_array<'de, D, const N: usize>(deserializer: D) -> Result<[f32; N], D::Error>
where
    D: Deserializer<'de>,
{
    let values = Vec::<f32>::deserialize(deserializer)?;
    if values.len() > N {
        return Err(serde::de::Error::custom(format!(
            "expected at most {N} f32 values, got {}",
            values.len()
        )));
    }
    let mut out = [0.0; N];
    for (idx, value) in values.into_iter().enumerate() {
        out[idx] = value;
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy)]
struct BetaCalibrator {
    a: f32,
    b: f32,
    c: f32,
    enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BetaCalibratorSnapshot {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub enabled: bool,
}

#[derive(Debug, Clone, Default)]
struct IsotonicCalibrator {
    thresholds: Vec<f32>,
    values: Vec<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IsotonicCalibratorSnapshot {
    pub thresholds: Vec<f32>,
    pub values: Vec<f32>,
}

#[derive(Debug, Clone, Default)]
struct TreeEnsemble {
    trees: Vec<BoostedTree>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TreeEnsembleSnapshot {
    pub trees: Vec<BoostedTree>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
pub struct BoostedTree {
    pub root_feature: usize,
    pub root_threshold: f32,
    pub left_feature: usize,
    pub left_threshold: f32,
    pub left_left_value: f32,
    pub left_right_value: f32,
    pub right_feature: usize,
    pub right_threshold: f32,
    pub right_left_value: f32,
    pub right_right_value: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetaFeatureWeight {
    pub name: &'static str,
    pub weight: f32,
    pub abs_weight: f32,
}

impl OnlineMetaCalibratorSnapshot {
    pub fn beta_enabled(&self) -> bool {
        self.beta.enabled
    }

    pub fn beta_coefficients(&self) -> (f32, f32, f32) {
        (self.beta.a, self.beta.b, self.beta.c)
    }

    pub fn isotonic_bins(&self) -> usize {
        self.isotonic.values.len()
    }

    pub fn tree_count(&self) -> usize {
        self.trees.trees.len()
    }

    pub fn tree_split_count(&self) -> usize {
        self.trees.trees.len() * 3
    }

    pub fn top_feature_weights(&self, limit: usize) -> Vec<MetaFeatureWeight> {
        let mut importance = [0.0f32; META_FEATURES];
        for tree in &self.trees.trees {
            let root_gain = tree.gain.max(0.0);
            importance[tree.root_feature.min(META_FEATURES - 1)] += root_gain;
            importance[tree.left_feature.min(META_FEATURES - 1)] += root_gain * 0.5;
            importance[tree.right_feature.min(META_FEATURES - 1)] += root_gain * 0.5;
        }
        let mut weights: Vec<MetaFeatureWeight> = self
            .weights
            .iter()
            .enumerate()
            .map(|(idx, weight)| MetaFeatureWeight {
                name: META_FEATURE_NAMES[idx],
                weight: *weight + importance[idx],
                abs_weight: weight.abs() + importance[idx],
            })
            .collect();
        weights.sort_by(|a, b| {
            b.abs_weight
                .partial_cmp(&a.abs_weight)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        weights.truncate(limit.min(weights.len()));
        weights
    }
}

impl Default for BetaCalibrator {
    fn default() -> Self {
        Self {
            a: 1.0,
            b: -1.0,
            c: 0.0,
            enabled: false,
        }
    }
}

impl Default for BetaCalibratorSnapshot {
    fn default() -> Self {
        BetaCalibrator::default().snapshot()
    }
}

impl BetaCalibrator {
    fn from_snapshot(snapshot: BetaCalibratorSnapshot) -> Self {
        Self {
            a: snapshot.a,
            b: snapshot.b,
            c: snapshot.c,
            enabled: snapshot.enabled,
        }
    }

    fn snapshot(&self) -> BetaCalibratorSnapshot {
        BetaCalibratorSnapshot {
            a: self.a,
            b: self.b,
            c: self.c,
            enabled: self.enabled,
        }
    }

    fn predict(&self, base_probability: f32) -> f32 {
        let p = base_probability.clamp(1.0e-6, 1.0 - 1.0e-6);
        if !self.enabled {
            return p;
        }
        let z = self.a * p.ln() + self.b * (1.0 - p).ln() + self.c;
        sigmoid(z).clamp(1.0e-6, 1.0 - 1.0e-6)
    }

    fn fit(samples: &[MetaTrainingSample]) -> Self {
        if samples.len() < BETA_CALIBRATOR_MIN_SAMPLES {
            return Self::default();
        }
        let mut beta = Self {
            enabled: true,
            ..Self::default()
        };
        let mut prepared = Vec::with_capacity(samples.len());
        for sample in samples {
            let p = sample.base_side_probability.clamp(1.0e-6, 1.0 - 1.0e-6);
            prepared.push((
                p,
                p.ln(),
                (1.0 - p).ln(),
                if sample.side_observed { 1.0 } else { 0.0 },
            ));
        }
        let n = samples.len() as f32;
        for _ in 0..BETA_CALIBRATOR_EPOCHS {
            let mut grad_a = 0.0;
            let mut grad_b = 0.0;
            let mut grad_c = 0.0;
            for (_, x_a, x_b, y) in &prepared {
                let pred = sigmoid(beta.a * x_a + beta.b * x_b + beta.c);
                let error = pred - *y;
                grad_a += error * *x_a;
                grad_b += error * *x_b;
                grad_c += error;
            }
            grad_a = grad_a / n + BETA_CALIBRATOR_L2 * (beta.a - 1.0);
            grad_b = grad_b / n + BETA_CALIBRATOR_L2 * (beta.b + 1.0);
            grad_c = grad_c / n + BETA_CALIBRATOR_L2 * beta.c;
            beta.a = (beta.a - BETA_CALIBRATOR_LR * grad_a)
                .clamp(-BETA_CALIBRATOR_COEFF_CLIP, BETA_CALIBRATOR_COEFF_CLIP);
            beta.b = (beta.b - BETA_CALIBRATOR_LR * grad_b)
                .clamp(-BETA_CALIBRATOR_COEFF_CLIP, BETA_CALIBRATOR_COEFF_CLIP);
            beta.c = (beta.c - BETA_CALIBRATOR_LR * grad_c)
                .clamp(-BETA_CALIBRATOR_COEFF_CLIP, BETA_CALIBRATOR_COEFF_CLIP);
        }

        let raw_log_loss = samples
            .iter()
            .map(|sample| binary_log_loss(sample.base_side_probability, sample.side_observed))
            .sum::<f32>()
            / n;
        let beta_log_loss = samples
            .iter()
            .zip(prepared.iter())
            .map(|(sample, (_, x_a, x_b, _))| {
                let calibrated = sigmoid(beta.a * *x_a + beta.b * *x_b + beta.c);
                binary_log_loss(calibrated, sample.side_observed)
            })
            .sum::<f32>()
            / n;
        if beta_log_loss + 1.0e-4 < raw_log_loss {
            beta
        } else {
            Self::default()
        }
    }
}

impl IsotonicCalibrator {
    fn from_snapshot(snapshot: IsotonicCalibratorSnapshot) -> Self {
        Self {
            thresholds: snapshot.thresholds,
            values: snapshot.values,
        }
    }

    fn snapshot(&self) -> IsotonicCalibratorSnapshot {
        IsotonicCalibratorSnapshot {
            thresholds: self.thresholds.clone(),
            values: self.values.clone(),
        }
    }

    fn is_empty(&self) -> bool {
        self.thresholds.is_empty() || self.values.is_empty()
    }

    fn fit(samples: &[MetaTrainingSample], beta: &BetaCalibrator) -> Self {
        if samples.len() < ISOTONIC_MIN_SAMPLES {
            return Self::default();
        }
        let mut pairs: Vec<(f32, f32)> = samples
            .iter()
            .map(|sample| {
                (
                    beta.predict(sample.base_side_probability)
                        .clamp(1.0e-6, 1.0 - 1.0e-6),
                    if sample.side_observed { 1.0 } else { 0.0 },
                )
            })
            .collect();
        pairs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        #[derive(Clone, Copy)]
        struct Block {
            max_x: f32,
            sum_y: f32,
            weight: f32,
        }

        let mut grouped: Vec<Block> = Vec::new();
        for (x, y) in pairs {
            if let Some(last) = grouped.last_mut() {
                if (last.max_x - x).abs() <= 1.0e-7 {
                    last.sum_y += y;
                    last.weight += 1.0;
                    continue;
                }
            }
            grouped.push(Block {
                max_x: x,
                sum_y: y,
                weight: 1.0,
            });
        }

        let total_weight = grouped.iter().map(|block| block.weight).sum::<f32>();
        let global_mean = (grouped.iter().map(|block| block.sum_y).sum::<f32>() / total_weight)
            .clamp(1.0e-4, 1.0 - 1.0e-4);
        let mut blocks: Vec<Block> = Vec::new();
        for group in grouped {
            blocks.push(group);
            while blocks.len() >= 2 {
                let n = blocks.len();
                let left_rate = blocks[n - 2].sum_y / blocks[n - 2].weight;
                let right_rate = blocks[n - 1].sum_y / blocks[n - 1].weight;
                if left_rate <= right_rate {
                    break;
                }
                let right = blocks.pop().expect("block exists");
                let left = blocks.pop().expect("block exists");
                blocks.push(Block {
                    max_x: right.max_x,
                    sum_y: left.sum_y + right.sum_y,
                    weight: left.weight + right.weight,
                });
            }
        }

        let mut thresholds = Vec::with_capacity(blocks.len());
        let mut values = Vec::with_capacity(blocks.len());
        for block in blocks {
            let observed = block.sum_y / block.weight;
            let shrunk = ((observed * block.weight) + (global_mean * ISOTONIC_SHRINKAGE))
                / (block.weight + ISOTONIC_SHRINKAGE);
            thresholds.push(block.max_x);
            values.push(shrunk.clamp(1.0e-4, 1.0 - 1.0e-4));
        }
        Self { thresholds, values }
    }

    fn predict(&self, base_probability: f32) -> f32 {
        if self.is_empty() {
            return base_probability;
        }
        let p = base_probability.clamp(1.0e-6, 1.0 - 1.0e-6);
        let idx = self
            .thresholds
            .partition_point(|threshold| *threshold <= p)
            .saturating_sub(1)
            .min(self.values.len().saturating_sub(1));
        self.values[idx]
    }
}

impl TreeEnsemble {
    fn from_snapshot(snapshot: TreeEnsembleSnapshot) -> Self {
        Self {
            trees: snapshot.trees,
        }
    }

    fn snapshot(&self) -> TreeEnsembleSnapshot {
        TreeEnsembleSnapshot {
            trees: self.trees.clone(),
        }
    }

    fn is_empty(&self) -> bool {
        self.trees.is_empty()
    }

    fn predict_logit_delta(&self, features: &MetaFeatures) -> f32 {
        self.trees.iter().map(|tree| tree.predict(features)).sum()
    }

    fn fit(
        samples: &[MetaTrainingSample],
        weights: &[f32; META_FEATURES],
        bias: f32,
        beta: &BetaCalibrator,
        isotonic: &IsotonicCalibrator,
    ) -> Self {
        if samples.len() < META_TREE_MIN_LEAF * 4 {
            return Self::default();
        }
        let stride = samples.len().div_ceil(META_TREE_MAX_TRAIN_SAMPLES).max(1);
        let train_indices: Vec<usize> = (0..samples.len()).step_by(stride).collect();
        if train_indices.len() < META_TREE_MIN_LEAF * 4 {
            return Self::default();
        }
        let mut trees = Vec::with_capacity(META_TREE_COUNT);
        for _ in 0..META_TREE_COUNT {
            let tree = fit_boosted_tree(
                samples,
                &train_indices,
                weights,
                bias,
                beta,
                isotonic,
                &trees,
            );
            if tree.gain <= 1.0e-6 {
                break;
            }
            trees.push(tree);
        }
        Self { trees }
    }
}

impl BoostedTree {
    fn predict(&self, features: &MetaFeatures) -> f32 {
        let root_value = features.values[self.root_feature.min(META_FEATURES - 1)];
        if root_value <= self.root_threshold {
            let value = features.values[self.left_feature.min(META_FEATURES - 1)];
            if value <= self.left_threshold {
                self.left_left_value
            } else {
                self.left_right_value
            }
        } else {
            let value = features.values[self.right_feature.min(META_FEATURES - 1)];
            if value <= self.right_threshold {
                self.right_left_value
            } else {
                self.right_right_value
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct SplitCandidate {
    feature: usize,
    threshold: f32,
    gain: f32,
    left_leaf: f32,
    right_leaf: f32,
}

#[derive(Debug, Clone, Copy)]
struct RootStumpDelta {
    feature: usize,
    threshold: f32,
    left_leaf: f32,
    right_leaf: f32,
}

impl RootStumpDelta {
    fn predict(self, features: &MetaFeatures) -> f32 {
        if features.values[self.feature.min(META_FEATURES - 1)] <= self.threshold {
            self.left_leaf
        } else {
            self.right_leaf
        }
    }
}

fn fit_boosted_tree(
    samples: &[MetaTrainingSample],
    train_indices: &[usize],
    weights: &[f32; META_FEATURES],
    bias: f32,
    beta: &BetaCalibrator,
    isotonic: &IsotonicCalibrator,
    prior_trees: &[BoostedTree],
) -> BoostedTree {
    let root = best_split(
        samples,
        train_indices,
        weights,
        bias,
        beta,
        isotonic,
        prior_trees,
        None,
    )
    .unwrap_or_default();
    if root.gain <= 0.0 {
        return BoostedTree::default();
    }

    let root_delta = RootStumpDelta {
        feature: root.feature,
        threshold: root.threshold,
        left_leaf: root.left_leaf,
        right_leaf: root.right_leaf,
    };

    let (left_indices, right_indices): (Vec<usize>, Vec<usize>) = train_indices
        .iter()
        .copied()
        .partition(|idx| samples[*idx].features.values[root.feature] <= root.threshold);

    let left = best_split(
        samples,
        &left_indices,
        weights,
        bias,
        beta,
        isotonic,
        prior_trees,
        Some(root_delta),
    );
    let right = best_split(
        samples,
        &right_indices,
        weights,
        bias,
        beta,
        isotonic,
        prior_trees,
        Some(root_delta),
    );

    let (left_feature, left_threshold, left_left_value, left_right_value, left_gain) =
        if let Some(left) = left {
            (
                left.feature,
                left.threshold,
                root.left_leaf + left.left_leaf,
                root.left_leaf + left.right_leaf,
                left.gain,
            )
        } else {
            (
                root.feature,
                root.threshold,
                root.left_leaf,
                root.left_leaf,
                0.0,
            )
        };
    let (right_feature, right_threshold, right_left_value, right_right_value, right_gain) =
        if let Some(right) = right {
            (
                right.feature,
                right.threshold,
                root.right_leaf + right.left_leaf,
                root.right_leaf + right.right_leaf,
                right.gain,
            )
        } else {
            (
                root.feature,
                root.threshold,
                root.right_leaf,
                root.right_leaf,
                0.0,
            )
        };

    BoostedTree {
        root_feature: root.feature,
        root_threshold: finite_or(root.threshold, 0.0),
        left_feature,
        left_threshold: finite_or(left_threshold, root.threshold),
        left_left_value: finite_or(clamp_tree_leaf(left_left_value), 0.0),
        left_right_value: finite_or(clamp_tree_leaf(left_right_value), 0.0),
        right_feature,
        right_threshold: finite_or(right_threshold, root.threshold),
        right_left_value: finite_or(clamp_tree_leaf(right_left_value), 0.0),
        right_right_value: finite_or(clamp_tree_leaf(right_right_value), 0.0),
        gain: finite_or(root.gain + 0.5 * (left_gain + right_gain), 0.0),
    }
}

fn finite_or(value: f32, fallback: f32) -> f32 {
    if value.is_finite() { value } else { fallback }
}

fn clamp_tree_leaf(value: f32) -> f32 {
    value.clamp(-META_TREE_VALUE_CLIP, META_TREE_VALUE_CLIP)
}

fn best_split(
    samples: &[MetaTrainingSample],
    indices: &[usize],
    weights: &[f32; META_FEATURES],
    bias: f32,
    beta: &BetaCalibrator,
    isotonic: &IsotonicCalibrator,
    prior_trees: &[BoostedTree],
    provisional_delta: Option<RootStumpDelta>,
) -> Option<SplitCandidate> {
    if indices.len() < META_TREE_MIN_LEAF * 2 {
        return None;
    }
    let parent = gradient_sums(
        samples,
        indices,
        weights,
        bias,
        beta,
        isotonic,
        prior_trees,
        provisional_delta,
        None,
    );
    let parent_gain = split_score(parent.0, parent.1);
    let mut best = SplitCandidate::default();
    for feature in 0..META_FEATURES {
        let thresholds = candidate_thresholds(samples, indices, feature);
        for threshold in thresholds {
            let left = gradient_sums(
                samples,
                indices,
                weights,
                bias,
                beta,
                isotonic,
                prior_trees,
                provisional_delta,
                Some((feature, threshold, true)),
            );
            let left_n = left.2;
            let right_n = indices.len().saturating_sub(left_n);
            if left_n < META_TREE_MIN_LEAF || right_n < META_TREE_MIN_LEAF {
                continue;
            }
            let right_g = parent.0 - left.0;
            let right_h = parent.1 - left.1;
            let gain = split_score(left.0, left.1) + split_score(right_g, right_h) - parent_gain;
            if gain > best.gain {
                best = SplitCandidate {
                    feature,
                    threshold,
                    gain,
                    left_leaf: leaf_value(left.0, left.1),
                    right_leaf: leaf_value(right_g, right_h),
                };
            }
        }
    }
    (best.gain > 0.0).then_some(best)
}

fn candidate_thresholds(
    samples: &[MetaTrainingSample],
    indices: &[usize],
    feature: usize,
) -> Vec<f32> {
    if indices.len() < 2 {
        return Vec::new();
    }
    let mut values = Vec::with_capacity(indices.len());
    for idx in indices {
        values.push(samples[*idx].features.values[feature]);
    }
    values.sort_by(|a, b| a.total_cmp(b));
    values.dedup_by(|a, b| (*a - *b).abs() <= 1.0e-6);
    if values.len() < 2 {
        return Vec::new();
    }

    const QUANTILES: [f32; 11] = [
        0.02, 0.05, 0.10, 0.20, 0.35, 0.50, 0.65, 0.80, 0.90, 0.95, 0.98,
    ];
    let mut thresholds = Vec::with_capacity(QUANTILES.len());
    for q in QUANTILES {
        let left_idx = ((values.len() - 2) as f32 * q).round() as usize;
        let left_idx = left_idx.min(values.len() - 2);
        let left = values[left_idx];
        let right = values[left_idx + 1];
        if (right - left).abs() > 1.0e-6 {
            thresholds.push(0.5 * (left + right));
        }
    }
    thresholds.dedup_by(|a, b| (*a - *b).abs() <= 1.0e-6);
    thresholds
}

fn gradient_sums(
    samples: &[MetaTrainingSample],
    indices: &[usize],
    weights: &[f32; META_FEATURES],
    bias: f32,
    beta: &BetaCalibrator,
    isotonic: &IsotonicCalibrator,
    prior_trees: &[BoostedTree],
    provisional_delta: Option<RootStumpDelta>,
    split: Option<(usize, f32, bool)>,
) -> (f32, f32, usize) {
    let mut sum_g = 0.0f32;
    let mut sum_h = 0.0f32;
    let mut count = 0usize;
    for idx in indices {
        let sample = &samples[*idx];
        if let Some((feature, threshold, want_left)) = split {
            let is_left = sample.features.values[feature] <= threshold;
            if is_left != want_left {
                continue;
            }
        }
        let p = predict_probability_with_components_and_delta(
            sample,
            weights,
            bias,
            beta,
            isotonic,
            prior_trees,
            provisional_delta,
        );
        let y = if sample.side_observed { 1.0 } else { 0.0 };
        sum_g += y - p;
        sum_h += (p * (1.0 - p)).max(1.0e-4);
        count += 1;
    }
    (sum_g, sum_h, count)
}

fn split_score(sum_g: f32, sum_h: f32) -> f32 {
    (sum_g * sum_g) / (sum_h + META_TREE_L2)
}

fn leaf_value(sum_g: f32, sum_h: f32) -> f32 {
    (META_TREE_LEARNING_RATE * sum_g / (sum_h + META_TREE_L2))
        .clamp(-META_TREE_VALUE_CLIP, META_TREE_VALUE_CLIP)
}

fn predict_probability_with_components_and_delta(
    sample: &MetaTrainingSample,
    weights: &[f32; META_FEATURES],
    bias: f32,
    beta: &BetaCalibrator,
    isotonic: &IsotonicCalibrator,
    trees: &[BoostedTree],
    provisional_delta: Option<RootStumpDelta>,
) -> f32 {
    let base = sample.base_side_probability.clamp(1.0e-6, 1.0 - 1.0e-6);
    let beta_base = beta.predict(base).clamp(1.0e-6, 1.0 - 1.0e-6);
    let calibrated_base = isotonic.predict(beta_base).clamp(1.0e-6, 1.0 - 1.0e-6);
    let linear_delta = bias
        + weights
            .iter()
            .zip(sample.features.values.iter())
            .map(|(w, f)| w * f)
            .sum::<f32>();
    let tree_delta = trees
        .iter()
        .map(|tree| tree.predict(&sample.features))
        .sum::<f32>();
    let provisional = provisional_delta
        .map(|delta| delta.predict(&sample.features))
        .unwrap_or(0.0);
    sigmoid(logit(calibrated_base) + linear_delta + tree_delta + provisional)
}

impl Default for OnlineMetaCalibrator {
    fn default() -> Self {
        Self {
            weights: [0.0; META_FEATURES],
            bias: 0.0,
            beta: BetaCalibrator::default(),
            isotonic: IsotonicCalibrator::default(),
            trees: TreeEnsemble::default(),
            updates: 0,
        }
    }
}

impl OnlineMetaCalibrator {
    pub fn from_snapshot(snapshot: OnlineMetaCalibratorSnapshot) -> Self {
        Self {
            weights: snapshot.weights,
            bias: snapshot.bias,
            beta: BetaCalibrator::from_snapshot(snapshot.beta),
            isotonic: IsotonicCalibrator::from_snapshot(snapshot.isotonic),
            trees: TreeEnsemble::from_snapshot(snapshot.trees),
            updates: snapshot.updates,
        }
    }

    pub fn snapshot(&self) -> OnlineMetaCalibratorSnapshot {
        OnlineMetaCalibratorSnapshot {
            weights: self.weights,
            bias: self.bias,
            beta: self.beta.snapshot(),
            isotonic: self.isotonic.snapshot(),
            trees: self.trees.snapshot(),
            updates: self.updates,
        }
    }

    fn linear_logit_delta(&self, features: &MetaFeatures) -> f32 {
        let dot = self
            .weights
            .iter()
            .zip(features.values.iter())
            .map(|(w, f)| w * f)
            .sum::<f32>();
        self.bias + dot
    }

    fn apply_logit_delta(&self, side_probability: f32, features: &MetaFeatures) -> f32 {
        let base = side_probability.clamp(1e-6, 1.0 - 1e-6);
        let beta_base = self.beta.predict(base).clamp(1.0e-6, 1.0 - 1.0e-6);
        let calibrated_base = self.isotonic.predict(beta_base).clamp(1.0e-6, 1.0 - 1.0e-6);
        let adjusted = logit(calibrated_base)
            + self.linear_logit_delta(features)
            + self.trees.predict_logit_delta(features);
        sigmoid(adjusted).clamp(1e-6, 1.0 - 1e-6)
    }

    pub fn predict_side_win_probability(
        &self,
        side_probability: f32,
        features: &MetaFeatures,
    ) -> f32 {
        if self.updates < META_CALIBRATOR_MIN_UPDATES
            && !self.beta.enabled
            && self.isotonic.is_empty()
            && self.trees.is_empty()
        {
            side_probability
        } else {
            self.apply_logit_delta(side_probability, features)
        }
    }

    pub fn record(&mut self, features: &MetaFeatures, side_observed: bool) {
        self.record_with_base_probability(features, 0.5, side_observed);
    }

    pub fn record_with_base_probability(
        &mut self,
        features: &MetaFeatures,
        base_side_probability: f32,
        side_observed: bool,
    ) {
        let p_hat = self.apply_logit_delta(base_side_probability, features);
        let target = if side_observed { 1.0 } else { 0.0 };
        let error = target - p_hat;
        for (w, f) in self.weights.iter_mut().zip(features.values.iter()) {
            *w = (*w + META_CALIBRATOR_LR * error * *f - META_CALIBRATOR_WEIGHT_DECAY * *w)
                .clamp(-META_CALIBRATOR_WEIGHT_CLIP, META_CALIBRATOR_WEIGHT_CLIP);
        }
        self.bias = (self.bias + META_CALIBRATOR_LR * error)
            .clamp(-META_CALIBRATOR_WEIGHT_CLIP, META_CALIBRATOR_WEIGHT_CLIP);
        self.updates = self.updates.saturating_add(1);
    }

    pub fn fit_batch(
        &mut self,
        samples: &[MetaTrainingSample],
        cfg: MetaTrainingConfig,
    ) -> MetaTrainingStats {
        if cfg.reset_before_fit {
            *self = Self::default();
        }
        if samples.is_empty() || cfg.epochs == 0 {
            return MetaTrainingStats {
                samples: samples.len(),
                epochs: cfg.epochs,
                updates: self.updates,
                log_loss: 0.0,
            };
        }

        let lr = cfg.learning_rate.max(1.0e-6);
        let l2 = cfg.l2.max(0.0);
        let clip = cfg.weight_clip.max(0.01);
        self.beta = BetaCalibrator::fit(samples);
        self.isotonic = IsotonicCalibrator::fit(samples, &self.beta);
        for _ in 0..cfg.epochs {
            for sample in samples {
                let p_hat = self.apply_logit_delta(sample.base_side_probability, &sample.features);
                let target = if sample.side_observed { 1.0 } else { 0.0 };
                let error = target - p_hat;
                for (w, f) in self.weights.iter_mut().zip(sample.features.values.iter()) {
                    *w = (*w + lr * error * *f - l2 * *w).clamp(-clip, clip);
                }
                self.bias = (self.bias + lr * error).clamp(-clip, clip);
                self.updates = self.updates.saturating_add(1);
            }
        }
        self.trees = TreeEnsemble::fit(
            samples,
            &self.weights,
            self.bias,
            &self.beta,
            &self.isotonic,
        );
        let log_loss = samples
            .iter()
            .map(|sample| {
                binary_log_loss(
                    self.predict_side_win_probability(
                        sample.base_side_probability,
                        &sample.features,
                    ),
                    sample.side_observed,
                )
            })
            .sum::<f32>()
            / samples.len() as f32;
        MetaTrainingStats {
            samples: samples.len(),
            epochs: cfg.epochs,
            updates: self.updates,
            log_loss,
        }
    }

    pub fn updates(&self) -> u32 {
        self.updates
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SkewBucket {
    pub taken: u32,
    pub yes_wins: u32,
}

#[derive(Debug, Clone, Default)]
pub struct SkewWinRateTable {
    buckets: [SkewBucket; SKEW_BINS],
}

impl SkewWinRateTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&mut self, market_mid: f32, predicted_yes: bool, resolved_yes: bool) {
        let idx = bucket_idx(market_mid);
        let b = &mut self.buckets[idx];
        b.taken = b.taken.saturating_add(1);
        if predicted_yes && resolved_yes || !predicted_yes && !resolved_yes {
            b.yes_wins = b.yes_wins.saturating_add(1);
        }
    }

    pub fn expected_side_win_rate(&self, market_mid: f32, predicted_yes: bool) -> f32 {
        let b = self.buckets[bucket_idx(market_mid)];
        if b.taken == 0 {
            return 0.5;
        }
        let p = if predicted_yes {
            b.yes_wins as f32 / b.taken as f32
        } else if b.taken == 0 {
            0.5
        } else {
            1.0 - (b.yes_wins as f32 / b.taken as f32)
        };
        p.clamp(0.0, 1.0)
    }

    pub fn mismatch_penalty(&self, market_mid: f32, predicted_yes: bool) -> f32 {
        let implied = if predicted_yes {
            market_mid.clamp(0.0, 1.0)
        } else {
            (1.0 - market_mid).clamp(0.0, 1.0)
        };
        let observed = self.expected_side_win_rate(market_mid, predicted_yes);
        (observed - implied).abs().clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ModelConfig {
    pub micro_dev_scale: f32,
    pub depth_full_at_shares: f32,
    pub book_weight: f32,
    pub spot_weight: f32,
    pub model_weight: f32,
    pub calibrated_p_min: f32,
    pub calibrated_p_max: f32,
    pub early_penalty_secs: f32,
    /// Exponent >1 softens the early-booking restriction; <1 punishes earlier.
    pub early_penalty_power: f32,
    /// Number of recent direction-score ticks used for stability.
    pub stability_window: usize,
    /// Enable online meta-calibration of `calibrated_p` using post-resolution outcomes.
    pub enable_meta_calibration: bool,
    /// Blend strength for time-of-day confidence/risk adjustment.
    pub time_of_day_weight: f32,
    /// Blend strength for cross-market volatility regime in confidence/risk.
    pub volatility_weight: f32,
    /// Blend strength for BTC spot whipsaw risk in `risk_score`.
    pub btc_whipsaw_risk_weight: f32,
    /// Blend strength for BTC path inefficiency in `risk_score`.
    pub btc_path_inefficiency_risk_weight: f32,
    /// Blend strength for short-term BTC reversal pressure in `risk_score`.
    pub btc_reversal_pressure_risk_weight: f32,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            micro_dev_scale: 0.6,
            depth_full_at_shares: 1500.0,
            book_weight: 0.4,
            spot_weight: 0.6,
            model_weight: 0.7,
            calibrated_p_min: 0.55,
            calibrated_p_max: 0.94,
            early_penalty_secs: 60.0,
            early_penalty_power: 1.4,
            stability_window: 10,
            enable_meta_calibration: true,
            time_of_day_weight: TIME_OF_DAY_REGIME_WEIGHT,
            volatility_weight: VOLATILITY_REGIME_WEIGHT,
            btc_whipsaw_risk_weight: BTC_WHIPSAW_RISK_WEIGHT,
            btc_path_inefficiency_risk_weight: BTC_PATH_INEFFICIENCY_RISK_WEIGHT,
            btc_reversal_pressure_risk_weight: BTC_REVERSAL_PRESSURE_RISK_WEIGHT,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ModelState {
    recent_mids: TimedRing,
    recent_dir: Ring,
    pub top3_imbalance: TimedRing,
    pub skew_table: SkewWinRateTable,
    vol_state: VolatilityState,
    tod_table: TimeOfDayTable,
    last_event_ts_ns: i64,
    market_mid_min: f32,
    market_mid_max: f32,
    meta_calibrator: OnlineMetaCalibrator,
    dir_markov: DirectionMarkov,
    pending_meta_features: Option<PendingMetaTrainingSample>,
}

impl Default for ModelState {
    fn default() -> Self {
        Self::new()
    }
}

impl ModelState {
    pub fn new() -> Self {
        Self::with_capacities(64, 16, 32)
    }

    pub fn with_capacities(
        recent_mids_cap: usize,
        recent_dir_cap: usize,
        imbalance_cap: usize,
    ) -> Self {
        Self {
            recent_mids: TimedRing::new(recent_mids_cap),
            recent_dir: Ring::new(recent_dir_cap),
            top3_imbalance: TimedRing::new(imbalance_cap),
            skew_table: SkewWinRateTable::new(),
            vol_state: VolatilityState::default(),
            tod_table: TimeOfDayTable::default(),
            last_event_ts_ns: 0,
            market_mid_min: f32::INFINITY,
            market_mid_max: f32::NEG_INFINITY,
            meta_calibrator: OnlineMetaCalibrator::default(),
            dir_markov: DirectionMarkov::default(),
            pending_meta_features: None,
        }
    }

    pub fn record_market_result(
        &mut self,
        market_mid: f32,
        predicted_yes: bool,
        resolved_yes: bool,
    ) {
        self.skew_table
            .record(market_mid, predicted_yes, resolved_yes);
        let hour_bucket = hour_bucket(self.last_event_ts_ns);
        self.tod_table
            .record(hour_bucket, predicted_yes, resolved_yes);
        let _ = self.vol_state.finalize_market(market_mid);
        if let Some(pending) = self.pending_meta_features.take() {
            let side_observed = if predicted_yes {
                resolved_yes
            } else {
                !resolved_yes
            };
            self.meta_calibrator.record_with_base_probability(
                &pending.features,
                pending.base_side_probability,
                side_observed,
            );
        }
        self.reset_market_rolling_state();
    }

    fn reset_market_rolling_state(&mut self) {
        self.recent_mids.clear();
        self.recent_dir.clear();
        self.top3_imbalance.clear();
        self.dir_markov = DirectionMarkov::default();
        self.market_mid_min = f32::INFINITY;
        self.market_mid_max = f32::NEG_INFINITY;
        self.pending_meta_features = None;
    }

    pub fn meta_calibrator_snapshot(&self) -> OnlineMetaCalibratorSnapshot {
        self.meta_calibrator.snapshot()
    }

    pub fn load_meta_calibrator_snapshot(&mut self, snapshot: OnlineMetaCalibratorSnapshot) {
        self.meta_calibrator = OnlineMetaCalibrator::from_snapshot(snapshot);
    }

    pub fn fit_meta_calibrator(
        &mut self,
        samples: &[MetaTrainingSample],
        cfg: MetaTrainingConfig,
    ) -> MetaTrainingStats {
        self.meta_calibrator.fit_batch(samples, cfg)
    }

    pub fn evaluate(
        &mut self,
        event: &ReplayEvent,
        spot: &SpotHistory,
        seconds_since_window_open: f32,
        cfg: &ModelConfig,
    ) -> ModelOutput {
        self.evaluate_detailed(event, spot, seconds_since_window_open, cfg)
            .output
    }

    pub fn evaluate_detailed(
        &mut self,
        event: &ReplayEvent,
        spot: &SpotHistory,
        seconds_since_window_open: f32,
        cfg: &ModelConfig,
    ) -> ModelEvaluation {
        self.last_event_ts_ns = event.ts_ns;
        self.vol_state.on_event(event);
        self.recent_mids.push(event.ts_ns, event.yes_mid);
        self.market_mid_min = self.market_mid_min.min(event.yes_mid);
        self.market_mid_max = self.market_mid_max.max(event.yes_mid);
        let observed_yes_range_so_far =
            if self.market_mid_min.is_finite() && self.market_mid_max.is_finite() {
                self.market_mid_max - self.market_mid_min
            } else {
                0.0
            };

        let book_imb3 = top_n_ofi(event, 3);
        self.top3_imbalance.push(event.ts_ns, book_imb3);

        let (delta_5s, delta_15s) = top3_imbalance_deltas(&self.top3_imbalance, event.ts_ns);

        let mut book_dir = direction_score(event, &self.recent_mids, spot, cfg.micro_dev_scale);
        book_dir.top3_delta_5s = delta_5s;
        book_dir.top3_delta_15s = delta_15s;
        let imbalance_boost = (0.3 * delta_5s + 0.2 * delta_15s).clamp(-1.0, 1.0);
        let spot_regime = spot_regime_features(event.ts_ns, spot);

        let spot_score = book_dir.spot_momentum;
        let direction_raw = (book_dir.composite + 0.12 * imbalance_boost).clamp(-1.0, 1.0);
        let direction_score =
            (cfg.book_weight * direction_raw + cfg.spot_weight * spot_score).clamp(-1.0, 1.0);
        let direction_side = direction_score >= 0.0;
        let markov_persistence = self.dir_markov.observe(direction_side);
        self.recent_dir.push(direction_score);
        let sequence = sequence_features(&self.recent_dir);

        let hour_bucket = hour_bucket(event.ts_ns);
        let predicted_side_edge = self
            .tod_table
            .expected_side_win_rate(hour_bucket, direction_side);
        let volatility_regime = (self.vol_state.atr_like() * 5.0).clamp(0.0, 1.0);
        let time_of_day_advantage = (predicted_side_edge - 0.5) * 2.0;
        let conf = confidence_score(
            &self.recent_dir,
            seconds_since_window_open,
            cfg.early_penalty_secs,
            cfg.early_penalty_power,
            cfg.stability_window,
            time_of_day_advantage,
            cfg.time_of_day_weight,
            volatility_regime,
            cfg.volatility_weight,
            markov_persistence,
        );
        let time_of_day_penalty = (0.5 - predicted_side_edge).max(0.0) * 2.0;
        let risk = risk_score_with_btc_weights(
            event,
            &self.recent_dir,
            &self.top3_imbalance,
            cfg.depth_full_at_shares,
            direction_side,
            observed_yes_range_so_far,
            self.skew_table
                .mismatch_penalty(event.yes_mid, direction_side),
            volatility_regime,
            time_of_day_penalty,
            cfg.time_of_day_weight,
            cfg.volatility_weight,
            cfg.btc_whipsaw_risk_weight,
            cfg.btc_path_inefficiency_risk_weight,
            cfg.btc_reversal_pressure_risk_weight,
            1.0 - markov_persistence,
            spot_regime,
        );

        let calibrated_yes_p = calibrated_p(
            direction_score,
            conf.composite,
            event.yes_mid,
            cfg.model_weight,
        );

        let side = direction_side;
        let side_p_pre_meta = if side {
            calibrated_yes_p
        } else {
            1.0 - calibrated_yes_p
        }
        .clamp(cfg.calibrated_p_min, cfg.calibrated_p_max);
        let meta_features = MetaFeatures::from_raw(
            direction_score,
            book_dir,
            conf,
            risk,
            sequence,
            volatility_regime,
            book_imb3,
            event.yes_mid,
            side_p_pre_meta,
            observed_yes_range_so_far,
            spot_regime,
            seconds_since_window_open,
        );
        self.pending_meta_features = Some(PendingMetaTrainingSample {
            features: meta_features,
            base_side_probability: side_p_pre_meta,
        });
        let mut side_p_post_meta = side_p_pre_meta;
        let mut calibrated_p = side_p_pre_meta;
        if cfg.enable_meta_calibration {
            let calibrated_side = self
                .meta_calibrator
                .predict_side_win_probability(side_p_pre_meta, &meta_features);
            side_p_post_meta = calibrated_side.clamp(cfg.calibrated_p_min, cfg.calibrated_p_max);
            calibrated_p = side_p_post_meta;
        }

        ModelEvaluation {
            output: ModelOutput {
                direction_score,
                confidence_score: conf.composite,
                calibrated_p,
                risk_score: risk.composite,
            },
            attribution: ModelAttribution {
                meta_features,
                direction: book_dir,
                confidence: conf,
                risk,
                sequence,
                book_imbalance_top3: book_imb3,
                spot_score,
                direction_raw,
                direction_side_is_yes: side,
                side_probability_pre_meta: side_p_pre_meta,
                side_probability_post_meta: side_p_post_meta,
                volatility_regime,
                time_of_day_edge: predicted_side_edge,
                observed_yes_range_so_far: observed_yes_range_so_far.clamp(0.0, 1.0),
                observed_range_high_cert_interaction: risk.observed_range_high_cert_risk,
                spot_regime,
                meta_calibrator_updates: self.meta_calibrator.updates(),
            },
        }
    }
}

pub fn microprice(yes_bid: f32, yes_ask: f32, bid_size: f32, ask_size: f32) -> f32 {
    if bid_size + ask_size <= 0.0 || yes_bid <= 0.0 || yes_ask <= 0.0 {
        return 0.5 * (yes_bid + yes_ask);
    }
    (yes_bid * ask_size + yes_ask * bid_size) / (bid_size + ask_size)
}

pub fn ofi(bid_size: f32, ask_size: f32) -> f32 {
    let s = bid_size + ask_size;
    if s <= 0.0 {
        return 0.0;
    }
    let raw = (bid_size - ask_size) / s;
    raw.clamp(-1.0, 1.0)
}

pub fn top_n_ofi(event: &ReplayEvent, n: usize) -> f32 {
    let n = n.min(event.bids.len());
    if n == 0 {
        return 0.0;
    }
    ofi(
        event.bids[..n].iter().map(|l| l.size).sum(),
        event.asks[..n].iter().map(|l| l.size).sum(),
    )
}

pub fn direction_score(
    event: &ReplayEvent,
    recent_mids: &TimedRing,
    spot: &SpotHistory,
    micro_dev_scale: f32,
) -> DirectionScore {
    let momentum_features = mid_momentum_features(event, recent_mids);
    let momentum = momentum_features.weighted;
    let ofi_v = top_n_ofi(event, 3);
    let spot_stack = spot_score_stack(event.ts_ns, spot);
    let spot_momentum = spot_stack.blended;

    let mp = microprice(
        event.yes_bid,
        event.yes_ask,
        event.bids[0].size,
        event.asks[0].size,
    );
    let spread = (event.yes_ask - event.yes_bid).max(1e-6);
    let microprice_dev = (((mp - event.yes_mid) / spread) * micro_dev_scale).clamp(-1.0, 1.0);
    let micro_divergence = (microprice_dev - spot_momentum).abs();
    let microprice_spot_alignment =
        (1.0 - (micro_divergence * 1.3).clamp(0.0, 1.0)).clamp(0.0, 1.0);
    let composite = (0.42 * momentum
        + 0.22 * ofi_v
        + 0.21 * microprice_dev
        + 0.15 * (microprice_spot_alignment * 2.0 - 1.0))
        .clamp(-1.0, 1.0);

    DirectionScore {
        momentum,
        momentum_10s: momentum_features.by_window[0],
        momentum_30s: momentum_features.by_window[1],
        momentum_60s: momentum_features.by_window[2],
        momentum_120s: momentum_features.by_window[3],
        momentum_300s: momentum_features.by_window[4],
        momentum_10_60_accel: (momentum_features.by_window[0] - momentum_features.by_window[2])
            .clamp(-1.0, 1.0),
        momentum_30_300_accel: (momentum_features.by_window[1] - momentum_features.by_window[4])
            .clamp(-1.0, 1.0),
        spot_momentum,
        spot_fast_momentum: spot_stack.fast,
        spot_broad_momentum: spot_stack.broad,
        spot_momentum_600s: spot_stack.momentum_600s,
        spot_momentum_900s: spot_stack.momentum_900s,
        spot_momentum_1800s: spot_stack.momentum_1800s,
        spot_momentum_3600s: spot_stack.momentum_3600s,
        spot_momentum_7200s: spot_stack.momentum_7200s,
        spot_momentum_14400s: spot_stack.momentum_14400s,
        spot_1h_4h_alignment: spot_stack.one_hour_four_hour_alignment,
        spot_ultra_trend_consistency: spot_stack.ultra_trend_consistency,
        spot_ultra_acceleration: spot_stack.ultra_acceleration,
        spot_fast_broad_alignment: spot_stack.alignment,
        spot_fast_long_alignment: spot_stack.fast_long_alignment,
        spot_broad_trend_consistency: spot_stack.broad_trend_consistency,
        spot_acceleration: spot_stack.acceleration,
        spot_broad_acceleration: spot_stack.broad_acceleration,
        ofi: ofi_v,
        microprice_dev,
        microprice_spot_alignment,
        top3_delta_5s: 0.0,
        top3_delta_15s: 0.0,
        composite,
    }
}

#[derive(Debug, Clone, Copy)]
struct MomentumFeatures {
    by_window: [f32; 5],
    weighted: f32,
}

fn mid_momentum_features(event: &ReplayEvent, recent_mids: &TimedRing) -> MomentumFeatures {
    let mut by_window = [0.0f32; 5];
    let mut numerator = 0.0f32;
    let mut denom = 0.0f32;
    for (idx, secs) in MOMENTUM_WINDOWS_SECONDS.iter().enumerate() {
        let target = event.ts_ns.saturating_sub(secs * NS_PER_SECOND);
        if let Some((_, past_mid)) = recent_mids.value_at_or_before(target) {
            let raw = ((event.yes_mid - past_mid) * 25.0).clamp(-1.0, 1.0);
            by_window[idx] = raw;
            numerator += MOMENTUM_WEIGHTS[idx] * raw;
            denom += MOMENTUM_WEIGHTS[idx];
        }
    }
    let weighted = if denom <= 0.0 {
        0.0
    } else {
        (numerator / denom).clamp(-1.0, 1.0)
    };
    MomentumFeatures {
        by_window,
        weighted,
    }
}

pub fn confidence_score(
    recent_dir_scores: &Ring,
    seconds_since_window_open: f32,
    early_decay_secs: f32,
    early_decay_power: f32,
    stability_window: usize,
    time_of_day_advantage: f32,
    time_of_day_weight: f32,
    volatility_regime: f32,
    volatility_weight: f32,
    markov_persistence: f32,
) -> ConfidenceScore {
    let window = stability_window.max(1);
    let std_v = recent_dir_scores.std_last_n(window);
    let stability = (1.0 - (std_v / 0.45).clamp(0.0, 1.0)).clamp(0.0, 1.0);

    let window_len = recent_dir_scores.len().min(window);
    let flips = recent_dir_scores.sign_flips_last_n(window_len) as f32;
    let denom = (window_len.saturating_sub(1).max(1)) as f32;
    let persistence = (1.0 - (flips / denom)).clamp(0.0, 1.0);

    let early_market_penalty = if early_decay_secs <= 0.0 {
        0.0
    } else if seconds_since_window_open >= early_decay_secs {
        0.0
    } else if seconds_since_window_open <= 0.0 {
        1.0
    } else {
        let ratio = (seconds_since_window_open / early_decay_secs).clamp(0.0, 1.0);
        1.0 - ratio.powf(early_decay_power.max(0.01))
    };

    let markov_persistence = markov_persistence.clamp(0.0, 1.0);
    let confidence_base = (0.65 * stability + 0.2 * persistence + 0.15 * markov_persistence)
        * (1.0 - early_market_penalty).clamp(0.0, 1.0);
    let volatility_adjust = (1.0 - volatility_weight * volatility_regime).clamp(0.0, 1.0);
    let time_of_day_multiplier = (1.0 + time_of_day_weight * time_of_day_advantage).clamp(0.5, 1.5);
    let composite = (confidence_base * volatility_adjust * time_of_day_multiplier).clamp(0.0, 1.0);

    ConfidenceScore {
        stability,
        sign_persistence: persistence,
        markov_persistence,
        early_market_penalty,
        time_of_day_advantage,
        composite,
    }
}

fn sequence_features(recent_dir_scores: &Ring) -> SequenceFeatures {
    let dir_mean_3 = recent_dir_scores.mean_last_n(3);
    let dir_mean_8 = recent_dir_scores.mean_last_n(8);
    let n8 = recent_dir_scores.len().min(8);
    let dir_flip_rate_8 = if n8 < 2 {
        0.0
    } else {
        recent_dir_scores.sign_flips_last_n(n8) as f32 / (n8 - 1) as f32
    }
    .clamp(0.0, 1.0);
    let dir_std_8 = recent_dir_scores.std_last_n(8).clamp(0.0, 1.0);
    let dir_abs_mean_8 = mean_abs_last_n(recent_dir_scores, 8).clamp(0.0, 1.0);
    let newest = recent_dir_scores.newest().unwrap_or(0.0);
    let oldest = if recent_dir_scores.len() >= 8 {
        let idx = (recent_dir_scores.head + recent_dir_scores.cap - recent_dir_scores.len())
            % recent_dir_scores.cap;
        recent_dir_scores.buf[idx]
    } else {
        recent_dir_scores.oldest().unwrap_or(newest)
    };
    SequenceFeatures {
        dir_mean_3,
        dir_mean_8,
        dir_slope_8: (newest - oldest).clamp(-1.0, 1.0),
        dir_flip_rate_8,
        dir_std_8,
        dir_abs_mean_8,
    }
}

fn mean_abs_last_n(values: &Ring, n: usize) -> f32 {
    let n = n.min(values.len());
    if n == 0 {
        return 0.0;
    }
    let start = values.len() - n;
    let sum: f32 = (start..values.len())
        .map(|i| values.buf[(values.head + values.cap - values.len() + i) % values.cap].abs())
        .sum();
    sum / n as f32
}

pub fn risk_score(
    event: &ReplayEvent,
    recent_dir_scores: &Ring,
    top3_imbalance: &TimedRing,
    depth_full_at_shares: f32,
    direction_side_is_yes: bool,
    observed_yes_range_so_far: f32,
    skew_penalty: f32,
    volatility_penalty: f32,
    time_of_day_penalty: f32,
    time_of_day_weight: f32,
    volatility_weight: f32,
    markov_reversal_risk: f32,
    spot_regime: SpotRegimeFeatures,
) -> RiskScore {
    risk_score_with_btc_weights(
        event,
        recent_dir_scores,
        top3_imbalance,
        depth_full_at_shares,
        direction_side_is_yes,
        observed_yes_range_so_far,
        skew_penalty,
        volatility_penalty,
        time_of_day_penalty,
        time_of_day_weight,
        volatility_weight,
        BTC_WHIPSAW_RISK_WEIGHT,
        BTC_PATH_INEFFICIENCY_RISK_WEIGHT,
        BTC_REVERSAL_PRESSURE_RISK_WEIGHT,
        markov_reversal_risk,
        spot_regime,
    )
}

fn risk_score_with_btc_weights(
    event: &ReplayEvent,
    recent_dir_scores: &Ring,
    top3_imbalance: &TimedRing,
    depth_full_at_shares: f32,
    direction_side_is_yes: bool,
    observed_yes_range_so_far: f32,
    skew_penalty: f32,
    volatility_penalty: f32,
    time_of_day_penalty: f32,
    time_of_day_weight: f32,
    volatility_weight: f32,
    btc_whipsaw_risk_weight: f32,
    btc_path_inefficiency_risk_weight: f32,
    btc_reversal_pressure_risk_weight: f32,
    markov_reversal_risk: f32,
    spot_regime: SpotRegimeFeatures,
) -> RiskScore {
    let flips = recent_dir_scores.sign_flips() as f32;
    let denom = (recent_dir_scores.len().saturating_sub(1).max(1)) as f32;
    let whipsaw = (flips / denom).clamp(0.0, 1.0);

    let (imbalance_delta_5, imbalance_delta_15) =
        top3_imbalance_deltas(top3_imbalance, event.ts_ns);
    let imbalance_turn = (imbalance_delta_5.abs() + imbalance_delta_15.abs()) * 0.5;

    let depth = top_n_depth(event, 5);
    let liquidity = (depth / depth_full_at_shares).clamp(0.0, 1.0);
    let path_risk = 1.0 - (2.0 * (event.yes_mid - 0.5).abs()).clamp(0.0, 1.0);
    let markov_reversal_risk = markov_reversal_risk.clamp(0.0, 1.0);
    let side_market_price = if direction_side_is_yes {
        event.yes_mid
    } else {
        1.0 - event.yes_mid
    };
    let range_high_cert_risk =
        observed_range_high_cert_risk(observed_yes_range_so_far, side_market_price);
    let btc_path_inefficiency_risk = spot_regime.path_inefficiency.clamp(0.0, 1.0);
    let btc_whipsaw_risk = spot_regime.whipsaw_score.clamp(0.0, 1.0);
    let base = 0.42 * whipsaw
        + 0.16 * path_risk
        + 0.16 * (1.0 - liquidity)
        + 0.10 * imbalance_turn
        + 0.16 * markov_reversal_risk;
    let skew_penalty = skew_penalty.clamp(0.0, 1.0);
    let volatility_penalty = volatility_penalty.clamp(0.0, 1.0);
    let time_of_day_penalty = time_of_day_penalty.clamp(0.0, 1.0);
    let composite = (base
        + 0.10 * skew_penalty
        + OBSERVED_RANGE_HIGH_CERT_RISK_WEIGHT * range_high_cert_risk
        + volatility_weight * volatility_penalty
        + time_of_day_weight * 0.5 * time_of_day_penalty
        + btc_whipsaw_risk_weight.clamp(0.0, 1.0) * btc_whipsaw_risk
        + btc_path_inefficiency_risk_weight.clamp(0.0, 1.0) * btc_path_inefficiency_risk
        + btc_reversal_pressure_risk_weight.clamp(0.0, 1.0)
            * spot_regime.reversal_pressure.clamp(0.0, 1.0))
    .clamp(0.0, 1.0);

    RiskScore {
        whipsaw,
        liquidity,
        path_risk,
        imbalance_turn: imbalance_turn.clamp(0.0, 1.0),
        markov_reversal_risk,
        skew_penalty,
        observed_range_high_cert_risk: range_high_cert_risk,
        btc_whipsaw_risk,
        btc_path_inefficiency_risk,
        btc_reversal_pressure: spot_regime.reversal_pressure.clamp(0.0, 1.0),
        volatility_penalty,
        time_of_day_penalty,
        composite,
    }
}

pub fn observed_range_high_cert_risk(
    observed_yes_range_so_far: f32,
    side_market_price: f32,
) -> f32 {
    let range_component = smoothstep(0.70, 0.95, observed_yes_range_so_far.clamp(0.0, 1.0));
    let high_cert_component = smoothstep(0.82, 0.94, side_market_price.clamp(0.0, 1.0));
    (range_component * high_cert_component).clamp(0.0, 1.0)
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    if edge1 <= edge0 {
        return if x >= edge1 { 1.0 } else { 0.0 };
    }
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn bucket_idx(market_mid: f32) -> usize {
    if market_mid <= 0.0 {
        return 0;
    }
    if market_mid >= 1.0 {
        return SKEW_BINS - 1;
    }
    let raw = (market_mid * SKEW_BINS as f32).floor() as usize;
    raw.min(SKEW_BINS - 1)
}

fn hour_bucket(ts_ns: i64) -> usize {
    if ts_ns <= 0 {
        return 0;
    }
    let seconds = (ts_ns / NS_PER_SECOND).rem_euclid(24 * 60 * 60);
    (seconds / 3600) as usize % HOUR_BINS
}

fn binary_log_loss(p: f32, observed: bool) -> f32 {
    let p = p.clamp(1.0e-6, 1.0 - 1.0e-6);
    if observed { -p.ln() } else { -(1.0 - p).ln() }
}

fn logit(p: f32) -> f32 {
    let p = p.clamp(1.0e-6, 1.0 - 1.0e-6);
    (p / (1.0 - p)).ln()
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub fn top_n_depth(event: &ReplayEvent, n: usize) -> f32 {
    let n = n.min(event.bids.len());
    let bid_d: f32 = event.bids[..n].iter().map(|l| l.size).sum();
    let ask_d: f32 = event.asks[..n].iter().map(|l| l.size).sum();
    0.5 * (bid_d + ask_d)
}

pub fn calibrated_p(direction: f32, confidence: f32, book_implied: f32, model_weight: f32) -> f32 {
    let edge = (direction * confidence * 0.4).clamp(-0.4, 0.4);
    let model = (0.5 + edge).clamp(0.0, 1.0);
    let blended = model_weight * model + (1.0 - model_weight) * book_implied;
    blended
}

#[derive(Debug, Clone, Copy, Default)]
struct SpotScoreStack {
    fast: f32,
    broad: f32,
    momentum_600s: f32,
    momentum_900s: f32,
    momentum_1800s: f32,
    momentum_3600s: f32,
    momentum_7200s: f32,
    momentum_14400s: f32,
    blended: f32,
    alignment: f32,
    fast_long_alignment: f32,
    one_hour_four_hour_alignment: f32,
    broad_trend_consistency: f32,
    ultra_trend_consistency: f32,
    acceleration: f32,
    broad_acceleration: f32,
    ultra_acceleration: f32,
}

fn spot_score_stack(ts_ns: i64, spot: &SpotHistory) -> SpotScoreStack {
    let fast = weighted_multi_tf_return(ts_ns, spot);
    let broad = weighted_broad_multi_tf_return(ts_ns, spot);
    let fast_score = score_spot_return(fast, 250.0);
    let broad_score = score_spot_return(broad, 80.0);
    let momentum_600s = score_spot_return(spot.simple_return(ts_ns, 600 * NS_PER_SECOND), 120.0);
    let momentum_900s = score_spot_return(spot.simple_return(ts_ns, 900 * NS_PER_SECOND), 100.0);
    let momentum_1800s = score_spot_return(spot.simple_return(ts_ns, 1800 * NS_PER_SECOND), 70.0);
    let momentum_3600s = score_spot_return(spot.simple_return(ts_ns, 3600 * NS_PER_SECOND), 50.0);
    let momentum_7200s = score_spot_return(spot.simple_return(ts_ns, 7200 * NS_PER_SECOND), 35.0);
    let momentum_14400s = score_spot_return(spot.simple_return(ts_ns, 14400 * NS_PER_SECOND), 25.0);
    let alignment = if fast_score.abs() < 0.02 || broad_score.abs() < 0.02 {
        0.0
    } else if fast_score.signum() == broad_score.signum() {
        1.0
    } else {
        -1.0
    };
    let fast_long_alignment = directional_alignment(fast_score, momentum_3600s);
    let one_hour_four_hour_alignment = directional_alignment(momentum_3600s, momentum_14400s);
    let broad_scores = [momentum_600s, momentum_900s, momentum_1800s, momentum_3600s];
    let ultra_scores = [momentum_3600s, momentum_7200s, momentum_14400s];
    let non_zero = broad_scores
        .iter()
        .filter(|score| score.abs() >= 0.02)
        .count();
    let positive = broad_scores.iter().filter(|score| **score >= 0.02).count();
    let negative = broad_scores.iter().filter(|score| **score <= -0.02).count();
    let broad_trend_consistency = if non_zero == 0 {
        0.0
    } else {
        (positive.max(negative) as f32 / non_zero as f32).clamp(0.0, 1.0)
    };
    let ultra_non_zero = ultra_scores
        .iter()
        .filter(|score| score.abs() >= 0.02)
        .count();
    let ultra_positive = ultra_scores.iter().filter(|score| **score >= 0.02).count();
    let ultra_negative = ultra_scores.iter().filter(|score| **score <= -0.02).count();
    let ultra_trend_consistency = if ultra_non_zero == 0 {
        0.0
    } else {
        (ultra_positive.max(ultra_negative) as f32 / ultra_non_zero as f32).clamp(0.0, 1.0)
    };
    let broad_acceleration = (momentum_600s - momentum_3600s).clamp(-1.0, 1.0);
    let ultra_score =
        (0.50 * momentum_3600s + 0.30 * momentum_7200s + 0.20 * momentum_14400s).clamp(-1.0, 1.0);
    let ultra_acceleration = (momentum_3600s - momentum_14400s).clamp(-1.0, 1.0);
    let blended = if alignment == 0.0 {
        fast_score
    } else if alignment > 0.0 {
        (0.74 * fast_score + 0.18 * broad_score + 0.08 * ultra_score).clamp(-1.0, 1.0)
    } else if directional_alignment(fast_score, ultra_score) < 0.0
        && ultra_trend_consistency >= 0.67
    {
        // Short impulses fighting a coherent 1h-4h regime are fragile in late favourites.
        (0.55 * fast_score + 0.30 * broad_score + 0.15 * ultra_score).clamp(-1.0, 1.0)
    } else {
        (0.62 * fast_score + 0.30 * broad_score + 0.08 * ultra_score).clamp(-1.0, 1.0)
    };
    SpotScoreStack {
        fast: fast_score,
        broad: broad_score,
        momentum_600s,
        momentum_900s,
        momentum_1800s,
        momentum_3600s,
        momentum_7200s,
        momentum_14400s,
        blended,
        alignment,
        fast_long_alignment,
        one_hour_four_hour_alignment,
        broad_trend_consistency,
        ultra_trend_consistency,
        acceleration: (fast_score - broad_score).clamp(-1.0, 1.0),
        broad_acceleration,
        ultra_acceleration,
    }
}

fn directional_alignment(a: f32, b: f32) -> f32 {
    if a.abs() < 0.02 || b.abs() < 0.02 {
        0.0
    } else if a.signum() == b.signum() {
        1.0
    } else {
        -1.0
    }
}

pub fn spot_score(ts_ns: i64, spot: &SpotHistory) -> f32 {
    spot_score_stack(ts_ns, spot).blended
}

fn spot_regime_features(now_ns: i64, spot: &SpotHistory) -> SpotRegimeFeatures {
    const WINDOW_SECS: i64 = 180;
    const STEP_SECS: i64 = 5;
    const MAX_SAMPLES: usize = (WINDOW_SECS / STEP_SECS) as usize + 1;

    if spot.is_empty() {
        return SpotRegimeFeatures::default();
    }

    let start_ns = now_ns - WINDOW_SECS * NS_PER_SECOND;
    let mut prices = [0.0f64; MAX_SAMPLES];
    let mut len = 0usize;
    let mut next_ns = start_ns;
    while next_ns <= now_ns && len < MAX_SAMPLES {
        if let Some(price) = spot.price_at_or_before(next_ns) {
            if price.is_finite() && price > 0.0 {
                prices[len] = price;
                len += 1;
            }
        }
        next_ns += STEP_SECS * NS_PER_SECOND;
    }
    if len < 8 {
        return SpotRegimeFeatures::default();
    }

    let first = prices[0];
    let last = prices[len - 1];
    if first <= 0.0 || last <= 0.0 {
        return SpotRegimeFeatures::default();
    }

    let mut path_abs = 0.0f64;
    let mut sumsq = 0.0f64;
    let mut returns_len = 0usize;
    let mut flips = 0usize;
    let mut prev_sign = 0i8;
    for idx in 1..len {
        let prev = prices[idx - 1];
        let next = prices[idx];
        if prev <= 0.0 || next <= 0.0 {
            continue;
        }
        let r = (next / prev).ln();
        if !r.is_finite() {
            continue;
        }
        path_abs += r.abs();
        sumsq += r * r;
        returns_len += 1;
        let sign = if r > 0.0 {
            1
        } else if r < 0.0 {
            -1
        } else {
            0
        };
        if sign != 0 {
            if prev_sign != 0 && sign != prev_sign {
                flips += 1;
            }
            prev_sign = sign;
        }
    }
    if returns_len < 7 || path_abs <= 0.0 {
        return SpotRegimeFeatures::default();
    }

    let net = (last / first).ln().abs();
    let path_efficiency = (net / path_abs).clamp(0.0, 1.0) as f32;
    let sign_flip_rate =
        (flips as f32 / returns_len.saturating_sub(1).max(1) as f32).clamp(0.0, 1.0);
    let realized_vol_180s_bps = ((sumsq / returns_len as f64).sqrt() * 10_000.0) as f32;

    let ret_30 = spot
        .simple_return(now_ns, 30 * NS_PER_SECOND)
        .unwrap_or(0.0);
    let ret_120 = spot
        .simple_return(now_ns, 120 * NS_PER_SECOND)
        .unwrap_or(0.0);
    let ret_180 = spot
        .simple_return(now_ns, 180 * NS_PER_SECOND)
        .unwrap_or(0.0);
    let reversal = ret_30.abs() * 10_000.0 >= 1.0
        && ret_120.abs().max(ret_180.abs()) * 10_000.0 >= 2.0
        && ret_30.signum() != ret_120.signum()
        && ret_30.signum() != ret_180.signum();

    let vol_component = (realized_vol_180s_bps / 7.5).clamp(0.0, 1.0);
    let path_inefficiency = (1.0 - path_efficiency).clamp(0.0, 1.0);
    let chop = path_inefficiency * vol_component;
    let reversal_flag = if reversal { 1.0 } else { 0.0 };
    let reversal_pressure = (0.7 * sign_flip_rate + 0.3 * reversal_flag).clamp(0.0, 1.0);
    let whipsaw_score =
        (0.50 * chop + 0.30 * sign_flip_rate + 0.15 * vol_component + 0.05 * reversal_flag)
            .clamp(0.0, 1.0);

    SpotRegimeFeatures {
        whipsaw_score,
        path_efficiency,
        path_inefficiency,
        sign_flip_rate,
        realized_vol_180s_bps,
        reversal_pressure,
    }
}

pub fn weighted_multi_tf_return(now_ns: i64, spot: &SpotHistory) -> Option<f64> {
    weighted_spot_return(
        now_ns,
        spot,
        &[(10, 0.45), (30, 0.25), (60, 0.15), (120, 0.10), (300, 0.05)],
    )
}

pub fn weighted_broad_multi_tf_return(now_ns: i64, spot: &SpotHistory) -> Option<f64> {
    weighted_spot_return(
        now_ns,
        spot,
        &[(600, 0.35), (900, 0.25), (1800, 0.25), (3600, 0.15)],
    )
}

fn weighted_spot_return(now_ns: i64, spot: &SpotHistory, windows: &[(i64, f64)]) -> Option<f64> {
    let mut sum = 0.0f64;
    let mut wsum = 0.0f64;
    for &(secs, weight) in windows {
        if let Some(r) = spot.simple_return(now_ns, secs * NS_PER_SECOND) {
            sum += weight * r;
            wsum += weight;
        }
    }
    if wsum <= 0.0 {
        return None;
    }
    Some(sum / wsum)
}

fn score_spot_return(r: Option<f64>, scale: f64) -> f32 {
    r.map(|value| (value * scale).clamp(-1.0, 1.0) as f32)
        .unwrap_or(0.0)
}

fn top3_imbalance_deltas(top3_imbalance: &TimedRing, now_ns: i64) -> (f32, f32) {
    let current = top3_imbalance.newest().map(|(_, v)| v).unwrap_or(0.0);
    let delta_5s = top3_imbalance.value_at_or_before(now_ns - 5 * NS_PER_SECOND);
    let delta_15s = top3_imbalance.value_at_or_before(now_ns - 15 * NS_PER_SECOND);
    let d5 = current - delta_5s.map_or(current, |(_, v)| v);
    let d15 = current - delta_15s.map_or(current, |(_, v)| v);
    (d5.clamp(-1.0, 1.0), d15.clamp(-1.0, 1.0))
}

pub fn entry_gate_satisfied(
    output: &ModelOutput,
    market_mid: f32,
    min_edge: f32,
    min_confidence: f32,
    max_risk: f32,
) -> bool {
    output.confidence_score >= min_confidence
        && output.risk_score <= max_risk
        && edge_vs_mid(output, market_mid).clamp(0.0, 1.0) >= min_edge
}

pub fn entry_gate_satisfied_for_side(
    output: &ModelOutput,
    market_mid: f32,
    yes_side: bool,
    min_edge: f32,
    min_confidence: f32,
    max_risk: f32,
) -> bool {
    output.confidence_score >= min_confidence
        && output.risk_score <= max_risk
        && side_edge_vs_mid(output, market_mid, yes_side).clamp(0.0, 1.0) >= min_edge
}

/// Edge of the predicted side versus the current implied probability.
///
/// `calibrated_p` is the predicted-side probability; the market side is
/// selected from `direction_score`.
pub fn edge_vs_mid(output: &ModelOutput, market_mid: f32) -> f32 {
    side_edge_vs_mid(output, market_mid, output.direction_score >= 0.0)
}

/// Edge of an explicit exposure side versus the current implied probability.
///
/// Use this for order-level gates. Event-level gates can use [`edge_vs_mid`],
/// but an order may be on the opposite side from the model's preferred side.
pub fn side_edge_vs_mid(output: &ModelOutput, market_mid: f32, yes_side: bool) -> f32 {
    let predicted_yes = output.direction_score >= 0.0;
    let side_mid = if yes_side {
        market_mid
    } else {
        1.0 - market_mid
    };
    let side_p = if yes_side == predicted_yes {
        output.calibrated_p
    } else {
        1.0 - output.calibrated_p
    };
    side_p - side_mid
}

#[cfg(test)]
mod tests {
    use super::*;
    use pm_types::{BookLevel, MarketId, ReplayFlags, SpotTick, tape::TAPE_DEPTH};

    fn evt(bid: f32, ask: f32, bid_size: f32, ask_size: f32, ts_ns: i64) -> ReplayEvent {
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
            ts_ns,
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

    fn mk_spot(base: f64, steps: usize, delta: f64) -> SpotHistory {
        let mut samples = Vec::with_capacity(steps);
        let mut price = base;
        for i in 0..steps {
            samples.push(SpotTick {
                ts_ns: i as i64 * NS_PER_SECOND,
                price,
                quantity: 0.0,
                is_buyer_maker: false,
            });
            price += delta;
        }
        SpotHistory::new(samples)
    }

    fn features_with_pair(x0: f32, x1: f32) -> MetaFeatures {
        let mut values = [0.0; META_FEATURES];
        values[0] = x0;
        values[1] = x1;
        MetaFeatures { values }
    }

    fn run_trending_ticks(state: &mut ModelState, spot: &SpotHistory) -> ModelOutput {
        let base_ns = 1_000_000_000;
        let mut out = ModelOutput::default();
        for i in 0..12 {
            out = state.evaluate(
                &evt(
                    0.53 + 0.004 * i as f32,
                    0.535 + 0.004 * i as f32,
                    200.0,
                    150.0,
                    base_ns + i as i64 * 1_000_000_000,
                ),
                spot,
                i as f32 + 70.0,
                &ModelConfig {
                    early_penalty_secs: 60.0,
                    ..ModelConfig::default()
                },
            );
        }
        out
    }

    #[test]
    fn model_output_bounds() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.2);
        let out = run_trending_ticks(&mut state, &spot);
        assert!((0.55..=0.94).contains(&out.calibrated_p));
        assert!((0.0..=1.0).contains(&out.confidence_score));
        assert!((0.0..=1.0).contains(&out.risk_score));
    }

    #[test]
    fn confidence_penalizes_early_window() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.0);
        let out_early = state.evaluate(
            &evt(0.51, 0.52, 100.0, 100.0, 1_000_000_000),
            &spot,
            1.0,
            &ModelConfig {
                early_penalty_secs: 60.0,
                ..ModelConfig::default()
            },
        );
        let out_late = state.evaluate(
            &evt(0.52, 0.53, 100.0, 100.0, 2_000_000_000),
            &spot,
            120.0,
            &ModelConfig {
                early_penalty_secs: 60.0,
                ..ModelConfig::default()
            },
        );
        assert!(out_late.confidence_score >= out_early.confidence_score);
    }

    #[test]
    fn entry_gate_matches_spec() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.15);
        let out = run_trending_ticks(&mut state, &spot);
        assert!(entry_gate_satisfied(&out, 0.5, 0.05, 0.68, 0.72));
    }

    #[test]
    fn entry_gate_is_side_oriented_for_no_side() {
        let out = ModelOutput {
            direction_score: -0.7,
            confidence_score: 0.9,
            calibrated_p: 0.64,
            risk_score: 0.1,
        };
        assert!(entry_gate_satisfied(&out, 0.58, 0.05, 0.68, 0.72));

        let bad = ModelOutput {
            direction_score: -0.7,
            confidence_score: 0.9,
            calibrated_p: 0.45,
            risk_score: 0.1,
        };
        assert!(!entry_gate_satisfied(&bad, 0.58, 0.05, 0.68, 0.72));
    }

    #[test]
    fn edge_vs_mid_is_side_oriented() {
        let out = ModelOutput {
            direction_score: -0.5,
            confidence_score: 0.7,
            calibrated_p: 0.62,
            risk_score: 0.2,
        };
        let edge = edge_vs_mid(&out, 0.58);
        assert!((edge - (0.62 - 0.42)).abs() < 1e-6);

        let out_yes = ModelOutput {
            direction_score: 0.5,
            confidence_score: 0.7,
            calibrated_p: 0.62,
            risk_score: 0.2,
        };
        let edge_yes = edge_vs_mid(&out_yes, 0.58);
        assert!((edge_yes - (0.62 - 0.58)).abs() < 1e-6);
    }

    #[test]
    fn explicit_side_gate_blocks_opposite_orders() {
        let out = ModelOutput {
            direction_score: -0.8,
            confidence_score: 0.9,
            calibrated_p: 0.65,
            risk_score: 0.1,
        };

        assert!(entry_gate_satisfied(&out, 0.60, 0.05, 0.68, 0.72));
        assert!(entry_gate_satisfied_for_side(
            &out, 0.60, false, 0.05, 0.68, 0.72
        ));
        assert!(!entry_gate_satisfied_for_side(
            &out, 0.60, true, 0.05, 0.68, 0.72
        ));
    }

    #[test]
    fn spot_weighted_multi_tf_weights() {
        let ns = 1_000_000_000;
        let spot = SpotHistory::new(vec![
            SpotTick {
                ts_ns: 0,
                price: 100.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 10 * ns,
                price: 100.5,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 30 * ns,
                price: 101.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 60 * ns,
                price: 101.5,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 120 * ns,
                price: 102.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 300 * ns,
                price: 103.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
        ]);
        let r = weighted_multi_tf_return(300 * ns, &spot).unwrap();
        assert!(r > 0.0, "expected positive multi-tf return");
    }

    #[test]
    fn spot_score_blends_broad_context_without_replacing_fast_signal() {
        let ns = 1_000_000_000;
        let spot = SpotHistory::new(vec![
            SpotTick {
                ts_ns: 0,
                price: 100.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 600 * ns,
                price: 101.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 900 * ns,
                price: 102.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 1800 * ns,
                price: 103.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 3300 * ns,
                price: 104.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: 3600 * ns,
                price: 105.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
        ]);
        assert!(weighted_broad_multi_tf_return(3600 * ns, &spot).unwrap() > 0.0);
        assert!(spot_score(3600 * ns, &spot) > 0.0);
    }

    #[test]
    fn skew_win_rate_table_records_and_penalizes_outlier_calibration() {
        let mut table = SkewWinRateTable::new();
        table.record(0.52, true, false);
        table.record(0.52, true, true);
        table.record(0.52, false, false);
        let expected_yes = table.expected_side_win_rate(0.52, true);
        assert!((expected_yes - 0.6666667).abs() < 1e-3);
        assert!((table.mismatch_penalty(0.52, true) - (expected_yes - 0.52).abs()).abs() < 1e-5);
    }

    #[test]
    fn risk_score_includes_skew_penalty() {
        let event = evt(0.50, 0.51, 100.0, 100.0, 2_000_000_000);
        let mut dir = Ring::new(4);
        dir.push(1.0);
        dir.push(-1.0);
        dir.push(1.0);
        dir.push(-1.0);
        let mut imbalance = TimedRing::new(4);
        imbalance.push(1_000_000_000, 0.2);
        imbalance.push(2_000_000_000, 0.4);
        let r0 = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.2,
            0.4,
            SpotRegimeFeatures::default(),
        )
        .composite;
        let r1 = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.2,
            0.4,
            SpotRegimeFeatures::default(),
        )
        .composite;
        assert!(r1 >= r0);
    }

    #[test]
    fn risk_score_penalizes_extreme_range_high_cert_side() {
        let event = evt(0.90, 0.91, 1000.0, 1000.0, 2_000_000_000);
        let mut dir = Ring::new(4);
        dir.push(1.0);
        dir.push(1.0);
        let mut imbalance = TimedRing::new(4);
        imbalance.push(2_000_000_000, 0.0);

        let calm = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.40,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            SpotRegimeFeatures::default(),
        );
        let extreme = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.98,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            SpotRegimeFeatures::default(),
        );

        assert!(extreme.observed_range_high_cert_risk > 0.5);
        assert!(extreme.composite > calm.composite);
    }

    #[test]
    fn risk_score_penalizes_btc_whipsaw_regime() {
        let event = evt(0.78, 0.79, 1000.0, 1000.0, 2_000_000_000);
        let mut dir = Ring::new(4);
        dir.push(1.0);
        dir.push(1.0);
        let mut imbalance = TimedRing::new(4);
        imbalance.push(2_000_000_000, 0.0);

        let calm = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.20,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            SpotRegimeFeatures::default(),
        );
        let choppy = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.20,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            SpotRegimeFeatures {
                whipsaw_score: 0.90,
                path_efficiency: 0.15,
                path_inefficiency: 0.85,
                sign_flip_rate: 0.80,
                realized_vol_180s_bps: 9.0,
                reversal_pressure: 0.95,
            },
        );

        assert!(choppy.btc_whipsaw_risk > calm.btc_whipsaw_risk);
        assert!(choppy.btc_path_inefficiency_risk > calm.btc_path_inefficiency_risk);
        assert!(choppy.btc_reversal_pressure > calm.btc_reversal_pressure);
        assert!(
            choppy.composite > calm.composite + 0.25,
            "calm={:?} choppy={:?}",
            calm,
            choppy
        );
    }

    #[test]
    fn risk_score_preserves_explicit_penalty_fields() {
        let event = evt(0.50, 0.51, 100.0, 100.0, 2_000_000_000);
        let mut dir = Ring::new(4);
        dir.push(1.0);
        dir.push(1.0);
        let mut imbalance = TimedRing::new(4);
        imbalance.push(2_000_000_000, 0.2);

        let risk = risk_score(
            &event,
            &dir,
            &imbalance,
            2000.0,
            true,
            0.0,
            0.11,
            0.22,
            0.33,
            0.44,
            0.55,
            0.66,
            SpotRegimeFeatures::default(),
        );

        assert!((risk.skew_penalty - 0.11).abs() < f32::EPSILON);
        assert!((risk.volatility_penalty - 0.22).abs() < f32::EPSILON);
        assert!((risk.time_of_day_penalty - 0.33).abs() < f32::EPSILON);
        assert!((risk.markov_reversal_risk - 0.66).abs() < f32::EPSILON);
    }

    #[test]
    fn meta_calibrator_learns_from_resolved_markets() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.0);
        let cfg = ModelConfig {
            enable_meta_calibration: true,
            ..ModelConfig::default()
        };

        let pre = state
            .evaluate(
                &evt(0.52, 0.53, 100.0, 100.0, 2_000_000_000),
                &spot,
                90.0,
                &cfg,
            )
            .calibrated_p;

        for i in 0..(META_CALIBRATOR_MIN_UPDATES + 4) {
            let _ = state.evaluate(
                &evt(0.52, 0.53, 100.0, 100.0, 2_000_000_000_i64 + i as i64),
                &spot,
                90.0,
                &cfg,
            );
            state.record_market_result(0.525, true, true);
        }

        let post = state
            .evaluate(
                &evt(0.52, 0.53, 100.0, 100.0, 3_000_000_000),
                &spot,
                90.0,
                &cfg,
            )
            .calibrated_p;

        assert!(state.meta_calibrator.updates() >= META_CALIBRATOR_MIN_UPDATES);
        assert!(post >= pre);
    }

    #[test]
    fn meta_calibrator_snapshot_roundtrips() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.0);
        let cfg = ModelConfig::default();
        let _ = state.evaluate(
            &evt(0.52, 0.53, 100.0, 100.0, 2_000_000_000),
            &spot,
            90.0,
            &cfg,
        );
        state.record_market_result(0.525, true, true);

        let snapshot = state.meta_calibrator_snapshot();
        let mut restored = ModelState::new();
        restored.load_meta_calibrator_snapshot(snapshot.clone());
        assert_eq!(restored.meta_calibrator_snapshot(), snapshot);
    }

    #[test]
    fn batch_meta_calibrator_learns_labeled_history() {
        let mut samples = Vec::new();
        for i in 0..80 {
            let side_observed = i % 2 == 0;
            let signal: f32 = if side_observed { 0.85 } else { -0.85 };
            let mut values = [0.0; META_FEATURES];
            values[..9].fill(signal);
            values[9] = 0.9;
            values[10] = 0.9;
            values[11] = 0.8;
            values[13] = 0.85;
            values[15] = 0.8;
            values[16] = 0.1;
            values[18] = 0.1;
            values[20] = 0.1;
            values[21] = 0.5;
            values[22] = 0.1;
            values[25] = 0.2;
            values[26] = signal;
            values[27] = signal;
            values[28] = signal;
            values[29] = signal.abs();
            values[31] = 0.72;
            let features = MetaFeatures { values };
            samples.push(MetaTrainingSample {
                features,
                market_idx: i as u32,
                base_side_probability: 0.55,
                side_observed,
            });
        }

        let mut calibrator = OnlineMetaCalibrator::default();
        let stats = calibrator.fit_batch(
            &samples,
            MetaTrainingConfig {
                epochs: 40,
                learning_rate: 0.06,
                ..MetaTrainingConfig::default()
            },
        );
        let good = calibrator.predict_side_win_probability(0.55, &samples[0].features);
        let bad = calibrator.predict_side_win_probability(0.55, &samples[1].features);

        assert_eq!(stats.samples, samples.len());
        assert!(stats.updates >= META_CALIBRATOR_MIN_UPDATES);
        assert!(stats.log_loss < 0.45, "log_loss={}", stats.log_loss);
        assert!(good > 0.70, "good={good}");
        assert!(bad < 0.40, "bad={bad}");
    }

    #[test]
    fn trained_meta_calibrator_snapshot_preserves_predictions() {
        let mut values = [0.0; META_FEATURES];
        values[..32].copy_from_slice(&[
            0.9, 0.8, 0.7, 0.5, 0.5, 0.2, 0.1, 0.7, 0.7, 0.9, 0.9, 0.8, 0.0, 0.85, 0.0, 0.8, 0.1,
            0.0, 0.1, 0.0, 0.1, 0.5, 0.1, 0.0, 0.0, 0.2, 0.8, 0.8, 0.4, 0.9, 0.0, 0.72,
        ]);
        let features = MetaFeatures { values };
        let samples = vec![
            MetaTrainingSample {
                features,
                market_idx: 0,
                base_side_probability: 0.56,
                side_observed: true,
            };
            24
        ];

        let mut calibrator = OnlineMetaCalibrator::default();
        calibrator.fit_batch(&samples, MetaTrainingConfig::default());
        let before = calibrator.predict_side_win_probability(0.56, &features);

        let restored = OnlineMetaCalibrator::from_snapshot(calibrator.snapshot());
        let after = restored.predict_side_win_probability(0.56, &features);
        assert!((before - after).abs() < 1e-6);
    }

    #[test]
    fn trained_meta_calibrator_snapshot_serializes_only_finite_tree_values() {
        let mut samples = Vec::new();
        for i in 0..300 {
            let mut values = [0.0; META_FEATURES];
            values[0] = i as f32 / 299.0;
            samples.push(MetaTrainingSample {
                features: MetaFeatures { values },
                market_idx: i as u32,
                base_side_probability: 0.55,
                side_observed: i >= 150,
            });
        }

        let mut calibrator = OnlineMetaCalibrator::default();
        calibrator.fit_batch(
            &samples,
            MetaTrainingConfig {
                epochs: 1,
                reset_before_fit: true,
                ..MetaTrainingConfig::default()
            },
        );
        let json = serde_json::to_string(&calibrator.snapshot()).unwrap();
        assert!(!json.contains("null"), "{json}");
        let _: OnlineMetaCalibratorSnapshot = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn meta_calibrator_uses_isotonic_and_tree_nonlinearity() {
        let mut samples = Vec::new();
        for i in 0..600 {
            let x0 = if i % 4 < 2 { 0.8 } else { -0.8 };
            let x1 = if i % 2 == 0 { 0.8 } else { -0.8 };
            let side_observed = x0 > 0.0 && x1 > 0.0 || x0 < 0.0 && x1 < 0.0;
            let mut values = [0.0; META_FEATURES];
            values[0] = x0;
            values[1] = x1;
            values[9] = 0.8;
            values[10] = 0.8;
            values[13] = 0.8;
            values[15] = 0.8;
            values[21] = 0.55;
            samples.push(MetaTrainingSample {
                features: MetaFeatures { values },
                market_idx: i as u32,
                base_side_probability: 0.56,
                side_observed,
            });
        }

        let mut calibrator = OnlineMetaCalibrator::default();
        calibrator.fit_batch(
            &samples,
            MetaTrainingConfig {
                epochs: 2,
                learning_rate: 0.005,
                l2: 0.01,
                weight_clip: 0.25,
                reset_before_fit: true,
            },
        );
        let snapshot = calibrator.snapshot();
        assert!(!snapshot.isotonic.values.is_empty());
        assert!(!snapshot.trees.trees.is_empty());
        assert!(
            snapshot
                .top_feature_weights(6)
                .iter()
                .any(|weight| weight.name == "direction_score_side"
                    || weight.name == "momentum_side")
        );
    }

    #[test]
    fn tree_child_splits_are_fit_after_root_stump_delta() {
        let mut samples = Vec::new();
        for i in 0..1_024 {
            let quadrant = i % 4;
            let x0 = if quadrant < 2 { -0.8 } else { 0.8 };
            let x1 = if quadrant % 2 == 0 { -0.8 } else { 0.8 };
            let positive_rate = match quadrant {
                0 => 0.10,
                1 => 0.35,
                2 => 0.65,
                _ => 0.90,
            };
            let side_observed = ((i / 4) % 100) as f32 / 100.0 < positive_rate;
            let mut values = [0.0; META_FEATURES];
            values[0] = x0;
            values[1] = x1;
            samples.push(MetaTrainingSample {
                features: MetaFeatures { values },
                market_idx: i as u32,
                base_side_probability: 0.50,
                side_observed,
            });
        }

        let train_indices = (0..samples.len()).collect::<Vec<_>>();
        let tree = fit_boosted_tree(
            &samples,
            &train_indices,
            &[0.0; META_FEATURES],
            0.0,
            &BetaCalibrator::default(),
            &IsotonicCalibrator::default(),
            &[],
        );

        assert!(tree.gain > 0.0);
        let low_low = tree.predict(&features_with_pair(-0.8, -0.8));
        let low_high = tree.predict(&features_with_pair(-0.8, 0.8));
        let high_low = tree.predict(&features_with_pair(0.8, -0.8));
        let high_high = tree.predict(&features_with_pair(0.8, 0.8));

        assert!(low_low < low_high, "{low_low} !< {low_high}");
        assert!(high_low < high_high, "{high_low} !< {high_high}");
        assert!(low_high < high_low, "{low_high} !< {high_low}");
    }

    #[test]
    fn meta_calibrator_enables_beta_when_base_probability_is_miscalibrated() {
        let mut samples = Vec::new();
        for i in 0..800 {
            let high_bucket = i < 400;
            let side_observed = if high_bucket { i % 5 != 0 } else { i % 5 == 0 };
            samples.push(MetaTrainingSample {
                features: MetaFeatures {
                    values: [0.0; META_FEATURES],
                },
                market_idx: i as u32,
                base_side_probability: if high_bucket { 0.60 } else { 0.40 },
                side_observed,
            });
        }

        let mut calibrator = OnlineMetaCalibrator::default();
        calibrator.fit_batch(
            &samples,
            MetaTrainingConfig {
                epochs: 1,
                learning_rate: 1.0e-6,
                reset_before_fit: true,
                ..MetaTrainingConfig::default()
            },
        );
        let snapshot = calibrator.snapshot();
        let high = calibrator.predict_side_win_probability(0.60, &samples[0].features);
        let low = calibrator.predict_side_win_probability(0.40, &samples[401].features);

        assert!(snapshot.beta_enabled());
        assert!(high > 0.60, "high={high}");
        assert!(low < 0.40, "low={low}");
    }

    #[test]
    fn meta_features_are_side_oriented() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.0);
        let cfg = ModelConfig {
            enable_meta_calibration: true,
            ..ModelConfig::default()
        };

        let yes_out = state.evaluate(
            &evt(0.52, 0.53, 100.0, 100.0, 2_000_000_000),
            &spot,
            90.0,
            &cfg,
        );
        state.record_market_result(0.525, true, true);

        let no_out = state.evaluate(
            &evt(0.48, 0.49, 100.0, 100.0, 3_000_000_000),
            &spot,
            90.0,
            &cfg,
        );
        state.record_market_result(0.475, false, false);

        assert!(state.meta_calibrator.updates() >= 2);
        assert!(yes_out.calibrated_p > 0.5 || no_out.calibrated_p > 0.5);
    }

    #[test]
    fn meta_features_include_side_price_edge_and_late_timing() {
        let features = MetaFeatures::from_raw(
            -0.25,
            DirectionScore::default(),
            ConfidenceScore {
                composite: 0.7,
                ..ConfidenceScore::default()
            },
            RiskScore::default(),
            SequenceFeatures::default(),
            0.0,
            0.0,
            0.20,
            0.86,
            0.42,
            SpotRegimeFeatures {
                whipsaw_score: 0.31,
                path_efficiency: 0.40,
                path_inefficiency: 0.60,
                sign_flip_rate: 0.50,
                realized_vol_180s_bps: 6.0,
                reversal_pressure: 0.25,
            },
            240.0,
        );

        let side_price_idx = META_FEATURE_NAMES
            .iter()
            .position(|name| *name == "side_market_price")
            .unwrap();
        assert!((features.values[side_price_idx] - 0.80).abs() < 1.0e-6);
        assert!((features.values[side_price_idx + 1] - 0.06).abs() < 1.0e-6);
        assert!((features.values[side_price_idx + 2] - 0.80).abs() < 1.0e-6);
        assert!((features.values[side_price_idx + 3] - 0.50).abs() < 1.0e-6);
        assert_eq!(features.values[side_price_idx + 4], 0.0);
        assert!((features.values[side_price_idx + 5] - 0.60).abs() < 1.0e-6);
        assert!((features.values[side_price_idx + 6] - 0.42).abs() < 1.0e-6);
        assert!((features.values[side_price_idx + 7] - 0.084).abs() < 1.0e-6);
        let regime_idx = META_FEATURE_NAMES
            .iter()
            .position(|name| *name == "btc_whipsaw_score")
            .unwrap();
        assert!((features.values[regime_idx] - 0.31).abs() < 1.0e-6);
        assert!((features.values[regime_idx + 1] - 0.40).abs() < 1.0e-6);
        assert!((features.values[regime_idx + 2] - 0.60).abs() < 1.0e-6);
        assert!((features.values[regime_idx + 4] - 0.50).abs() < 1.0e-6);
        assert_eq!(features.values[regime_idx + 6], 0.0);
    }

    #[test]
    fn spot_stack_includes_ultra_context() {
        let ns = |secs: i64| secs * NS_PER_SECOND;
        let spot = SpotHistory::new(vec![
            SpotTick {
                ts_ns: ns(0),
                price: 100.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: ns(3600),
                price: 101.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: ns(7200),
                price: 102.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: ns(10800),
                price: 106.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
            SpotTick {
                ts_ns: ns(14400),
                price: 110.0,
                quantity: 0.0,
                is_buyer_maker: false,
            },
        ]);

        let stack = spot_score_stack(ns(14400), &spot);
        assert!(stack.momentum_7200s > 0.0);
        assert!(stack.momentum_14400s > 0.0);
        assert_eq!(stack.one_hour_four_hour_alignment, 1.0);
        assert_eq!(stack.ultra_trend_consistency, 1.0);
    }

    #[test]
    fn meta_features_deserialize_older_short_vectors_with_zero_padding() {
        let json = format!(
            "{{\"values\":[{}]}}",
            (0..40).map(|i| i.to_string()).collect::<Vec<_>>().join(",")
        );
        let features: MetaFeatures = serde_json::from_str(&json).unwrap();
        assert_eq!(features.values[0], 0.0);
        assert_eq!(features.values[39], 39.0);
        assert_eq!(features.values[40], 0.0);
        assert_eq!(features.values[META_FEATURES - 1], 0.0);
    }

    #[test]
    fn time_of_day_bias_increases_with_historical_edge() {
        let mut with_bias = ModelState::new();
        let mut without_bias = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.0);
        let cfg = ModelConfig {
            time_of_day_weight: 1.0,
            volatility_weight: 0.0,
            ..ModelConfig::default()
        };

        let good_hour_ts = 10_000_000_000; // ~02:46 UTC
        let event = evt(0.52, 0.53, 100.0, 100.0, good_hour_ts);
        for _ in 0..6 {
            with_bias
                .tod_table
                .record(hour_bucket(good_hour_ts), true, true);
        }
        let conf_biased = with_bias
            .evaluate(&event, &spot, 30.0, &cfg)
            .confidence_score;
        let conf_plain = without_bias
            .evaluate(&event, &spot, 30.0, &cfg)
            .confidence_score;
        assert!(
            conf_biased >= conf_plain,
            "expected bias table to improve confidence"
        );
    }

    #[test]
    fn volatility_state_tracks_recent_markets() {
        let mut state = ModelState::new();
        let spot = mk_spot(100.0, 20, 0.0);
        let cfg = ModelConfig::default();
        let _ = state.evaluate(
            &evt(0.55, 0.56, 100.0, 100.0, 1_000_000_000),
            &spot,
            10.0,
            &cfg,
        );
        state.record_market_result(0.52, true, true);
        let _ = state.evaluate(
            &evt(0.57, 0.58, 100.0, 100.0, 2_000_000_000),
            &spot,
            10.0,
            &cfg,
        );
        state.record_market_result(0.54, false, false);
        let atr = state.vol_state.atr_like();
        assert!(atr >= 0.0);
    }
}
