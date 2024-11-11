use rust_decimal::Decimal;

use crate::level::Level;

#[derive(Debug, Clone)]
pub struct OrderbookMetrics {
    /// Quote imbalance ratio (-1 to 1), positive values indicate more bids
    pub quote_imbalance: Decimal,
    /// Mid price between best bid and ask
    pub mid_price: Decimal,
    /// Absolute spread (ask - bid)
    pub spread: Decimal,
    /// Spread as percentage of mid price
    ///
    /// The spread_percentage represents the bid-ask spread expressed
    /// as a percentage of the mid price.
    pub spread_percentage: Decimal,
    /// Estimated price impact for a market buy
    pub price_impact_buy: Decimal,
    /// Estimated price impact for a market sell
    pub price_impact_sell: Decimal,
}

// Shared implementation for metric calculation
pub trait MetricsCalculator {
    fn calculate_metrics_internal(
        &self,
        bid_sizes: Vec<Decimal>,
        ask_sizes: Vec<Decimal>,
        bid_prices: Vec<Decimal>,
        ask_prices: Vec<Decimal>,
    ) -> OrderbookMetrics {
        // Calculate mid price
        let mid_price = match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => (bid.price + ask.price) / Decimal::TWO,
            _ => Decimal::ZERO,
        };

        // Calculate quote imbalance
        let bid_value: Decimal = bid_sizes.iter().zip(bid_prices.iter()).map(|(&size, &price)| size * price).sum();
        let ask_value: Decimal = ask_sizes.iter().zip(ask_prices.iter()).map(|(&size, &price)| size * price).sum();
        let total_value = bid_value + ask_value;
        let quote_imbalance = if total_value > Decimal::ZERO { (bid_value - ask_value) / total_value } else { Decimal::ZERO };

        // Calculate spread
        let spread = match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => ask.price - bid.price,
            _ => Decimal::ZERO,
        };

        // Calculate spread percentage
        let spread_percentage = if mid_price > Decimal::ZERO { spread / mid_price * Decimal::ONE_HUNDRED } else { Decimal::ZERO };

        // Calculate price impact
        let price_impact_buy = if !ask_prices.is_empty() {
            (ask_prices[ask_prices.len() - 1] - mid_price) / mid_price * Decimal::ONE_HUNDRED
        } else {
            Decimal::ZERO
        };

        let price_impact_sell = if !bid_prices.is_empty() {
            (mid_price - bid_prices[bid_prices.len() - 1]) / mid_price * Decimal::ONE_HUNDRED
        } else {
            Decimal::ZERO
        };

        OrderbookMetrics { quote_imbalance, mid_price, spread, spread_percentage, price_impact_buy, price_impact_sell }
    }

    fn best_bid(&self) -> Option<Level>;
    fn best_ask(&self) -> Option<Level>;
}
