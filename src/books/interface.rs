use crate::{decimals::decimal_type::DecimalType, event::Event, level::Level, metrics::OrderbookMetrics};

pub trait OrderBook<V: DecimalType> {
    /// Process an incoming event
    fn process(&mut self, event: Event<V>);
    /// Get the current best bid
    fn best_bid(&mut self) -> Option<Level<V>>;
    /// Get the current best ask
    fn best_ask(&mut self) -> Option<Level<V>>;
    /// Calculate orderbook metrics up to specified depth
    fn calculate_metrics(&self, depth: usize) -> OrderbookMetrics<V>;
}
