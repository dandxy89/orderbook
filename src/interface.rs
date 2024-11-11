use crate::{event::Event, level::Level, metrics::OrderbookMetrics};

pub trait OrderBook {
    /// Process an incoming event
    fn process(&mut self, event: Event);
    /// Get the current best bid
    fn best_bid(&mut self) -> Option<Level>;
    /// Get the current best ask
    fn best_ask(&mut self) -> Option<Level>;
    /// Calculate orderbook metrics up to specified depth
    fn calculate_metrics(&self, depth: usize) -> OrderbookMetrics;
}

