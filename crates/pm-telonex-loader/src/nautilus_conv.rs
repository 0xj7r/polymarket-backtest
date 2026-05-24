//! Conversion adapters from internal `ReplayEvent` -> Nautilus model types.
//!
//! Phase 4 deliverable: prove the Nautilus crates link in our workspace AND
//! that we can produce engine-ready `QuoteTick`s from the loader output. The
//! actual `BacktestEngine` plumbing lives one layer up in `pm-app`.

use nautilus_core::UnixNanos;
use nautilus_model::data::QuoteTick;
use nautilus_model::identifiers::InstrumentId;
use nautilus_model::types::{Price, Quantity};
use pm_types::ReplayEvent;

/// Polymarket prices are bounded `[0, 1]` so 4 decimal places (1 bp ticks) is
/// generous; sizes are share counts that we keep at 2 dp.
pub const PRICE_PRECISION: u8 = 4;
pub const SIZE_PRECISION: u8 = 2;

/// Build a Nautilus `InstrumentId` for a Polymarket binary outcome.
///
/// Symbol form: `BTCUP-5M-{epoch_seconds}-{side}` (compact, ASCII, distinct
/// per market). Venue is hardcoded to `POLYMARKET`.
pub fn polymarket_instrument_id(symbol: &str) -> InstrumentId {
    InstrumentId::from(format!("{symbol}.POLYMARKET").as_str())
}

/// Convert top-of-book of a `ReplayEvent` to a Nautilus `QuoteTick`.
///
/// `ts_init` is set equal to `ts_event` for historical replay; in live mode the
/// data client overrides this with the wall clock.
pub fn to_quote_tick(event: &ReplayEvent, instrument_id: InstrumentId) -> QuoteTick {
    let ts = UnixNanos::from(event.ts_ns as u64);
    QuoteTick::new(
        instrument_id,
        Price::new(event.yes_bid.max(1e-4) as f64, PRICE_PRECISION),
        Price::new(event.yes_ask.max(1e-4) as f64, PRICE_PRECISION),
        Quantity::new(event.bids[0].size.max(0.0) as f64, SIZE_PRECISION),
        Quantity::new(event.asks[0].size.max(0.0) as f64, SIZE_PRECISION),
        ts,
        ts,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use pm_types::{BookLevel, MarketId, ReplayFlags, tape::TAPE_DEPTH};

    fn evt(bid: f32, ask: f32) -> ReplayEvent {
        let mut bids = [BookLevel::default(); TAPE_DEPTH];
        let mut asks = [BookLevel::default(); TAPE_DEPTH];
        bids[0] = BookLevel {
            price: bid,
            size: 200.0,
        };
        asks[0] = BookLevel {
            price: ask,
            size: 150.0,
        };
        ReplayEvent {
            ts_ns: 1_778_500_800_000_000_000,
            market_id: MarketId(7),
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
    fn quote_tick_round_trips_prices_and_sizes() {
        let iid = polymarket_instrument_id("BTCUP-5M-1778587500-UP");
        let q = to_quote_tick(&evt(0.50, 0.51), iid);
        // 0.50 with 4dp = 0.5000.
        assert!((q.bid_price.as_f64() - 0.5000).abs() < 1e-6);
        assert!((q.ask_price.as_f64() - 0.5100).abs() < 1e-6);
        assert!((q.bid_size.as_f64() - 200.00).abs() < 1e-6);
        assert!((q.ask_size.as_f64() - 150.00).abs() < 1e-6);
        assert_eq!(u64::from(q.ts_event), 1_778_500_800_000_000_000);
        assert_eq!(
            q.instrument_id,
            polymarket_instrument_id("BTCUP-5M-1778587500-UP")
        );
    }
}
