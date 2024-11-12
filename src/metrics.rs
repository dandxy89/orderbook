use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
};

use crate::{decimals::decimal_type::DecimalType, level::Level};

#[derive(Debug, Clone)]
pub struct OrderbookMetrics<V: DecimalType> {
    /// Quote imbalance ratio (-1 to 1), positive values indicate more bids
    pub quote_imbalance: V,
    /// Mid price between best bid and ask
    pub mid_price: V,
    /// Absolute spread (ask - bid)
    pub spread: V,
    /// Spread as percentage of mid price
    ///
    /// The spread_percentage represents the bid-ask spread expressed
    /// as a percentage of the mid price.
    pub spread_percentage: V,
    /// Estimated price impact for a market buy
    pub price_impact_buy: V,
    /// Estimated price impact for a market sell
    pub price_impact_sell: V,
}

// Shared implementation for metric calculation
pub trait MetricsCalculator<V>
where
    V: DecimalType + Sub<Output = V> + Add<Output = V> + Mul<Output = V> + Div<Output = V> + PartialOrd + Sum + Copy,
{
    fn calculate_metrics_internal(
        &self,
        bid_sizes: Vec<V>,
        ask_sizes: Vec<V>,
        bid_prices: Vec<V>,
        ask_prices: Vec<V>,
    ) -> OrderbookMetrics<V> {
        // Calculate mid price
        let mid_price = match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => (bid.price + ask.price) / V::TWO,
            _ => V::ZERO,
        };

        // Calculate quote imbalance
        let bid_value: V = bid_sizes.iter().zip(bid_prices.iter()).map(|(&size, &price)| size * price).sum();
        let ask_value: V = ask_sizes.iter().zip(ask_prices.iter()).map(|(&size, &price)| size * price).sum();
        let total_value = bid_value + ask_value;
        let quote_imbalance = if total_value > V::ZERO { (bid_value - ask_value) / total_value } else { V::ZERO };

        // Calculate spread
        let spread = match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => ask.price - bid.price,
            _ => V::ZERO,
        };

        // Calculate spread percentage
        let spread_percentage = if mid_price > V::ZERO { spread / mid_price * V::ONE_HUNDRED } else { V::ZERO };

        // Calculate price impact
        let price_impact_buy = if !ask_prices.is_empty() {
            (ask_prices[ask_prices.len() - 1] - mid_price) / mid_price * V::ONE_HUNDRED
        } else {
            V::ZERO
        };

        let price_impact_sell = if !bid_prices.is_empty() {
            (mid_price - bid_prices[bid_prices.len() - 1]) / mid_price * V::ONE_HUNDRED
        } else {
            V::ZERO
        };

        OrderbookMetrics { quote_imbalance, mid_price, spread, spread_percentage, price_impact_buy, price_impact_sell }
    }

    fn best_bid(&self) -> Option<Level<V>>;
    fn best_ask(&self) -> Option<Level<V>>;
}
