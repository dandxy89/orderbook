use std::{
    collections::BTreeMap,
    fmt::Debug,
    iter::Sum,
    ops::{Add, Div, Mul, Sub, SubAssign},
};

use rust_decimal::Decimal;

use crate::{
    books::interface::OrderBook,
    decimals::{decimal_type::DecimalType, fixed_decimal::FixedDecimal},
    event::Event,
    event_kind::EventKind,
    level::Level,
    metrics::{MetricsCalculator, OrderbookMetrics},
    side::Side,
};

#[derive(Debug)]
pub struct BTreeOrderBook<V>
where
    V: Debug + DecimalType,
{
    best_bid: Option<Level<V>>,
    best_ask: Option<Level<V>>,
    bids: BTreeMap<V, V>,
    asks: BTreeMap<V, V>,
    ts: i64,
    sequence_id: u64,
}

impl<V> OrderBook<V> for BTreeOrderBook<V>
where
    V: Debug
        + Ord
        + Copy
        + DecimalType
        + SubAssign
        + PartialOrd
        + Sub<Output = V>
        + SubAssign
        + Add<Output = V>
        + Mul<Output = V>
        + Div<Output = V>
        + Sum,
{
    fn process(&mut self, event: Event<V>) {
        let ts = event.timestamp;
        if ts < self.ts {
            return;
        }

        if event.sequence_id == 0
            || self.sequence_id == 0
            || event.sequence_id == self.sequence_id
            || event.sequence_id > self.sequence_id
        {
            self.ts = ts;

            match event.kind {
                EventKind::Trade => self.process_trade(event),
                EventKind::BBO => self.process_bbo(event),
                EventKind::L2 => self.process_l2(event),
            }
        }
    }

    fn best_bid(&mut self) -> Option<Level<V>> {
        self.best_bid
    }

    fn best_ask(&mut self) -> Option<Level<V>> {
        self.best_ask
    }

    fn calculate_metrics(&self, depth: usize) -> OrderbookMetrics<V> {
        let mut bid_sizes = Vec::with_capacity(depth);
        let mut ask_sizes = Vec::with_capacity(depth);
        let mut bid_prices = Vec::with_capacity(depth);
        let mut ask_prices = Vec::with_capacity(depth);

        // Collect bid data (in reverse order for descending prices)
        for (price, &size) in self.bids.iter().rev().take(depth) {
            bid_sizes.push(size);
            bid_prices.push(*price);
        }

        // Collect ask data
        for (price, &size) in self.asks.iter().take(depth) {
            ask_sizes.push(size);
            ask_prices.push(*price);
        }

        self.calculate_metrics_internal(bid_sizes, ask_sizes, bid_prices, ask_prices)
    }
}

impl<V> MetricsCalculator<V> for BTreeOrderBook<V>
where
    V: Debug
        + Copy
        + DecimalType
        + SubAssign
        + PartialOrd
        + Sub<Output = V>
        + SubAssign
        + Add<Output = V>
        + Mul<Output = V>
        + Div<Output = V>
        + Sum,
{
    fn best_bid(&self) -> Option<Level<V>> {
        self.best_bid
    }

    fn best_ask(&self) -> Option<Level<V>> {
        self.best_ask
    }
}

impl Default for BTreeOrderBook<Decimal> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Default for BTreeOrderBook<FixedDecimal> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl<V> BTreeOrderBook<V>
where
    V: Debug + DecimalType + SubAssign + PartialEq + PartialOrd + Ord + Copy,
{
    pub fn new() -> Self {
        Self { best_bid: None, best_ask: None, bids: BTreeMap::new(), asks: BTreeMap::new(), ts: 0, sequence_id: 0 }
    }

    fn process_l2(&mut self, event: Event<V>) {
        self.sequence_id = event.sequence_id;
        let (book, best_price) = match event.side {
            Side::Buy => (&mut self.bids, &mut self.best_bid),
            Side::Sell => (&mut self.asks, &mut self.best_ask),
        };
        if event.size == V::ZERO {
            book.remove(&event.price);
        } else {
            book.insert(event.price, event.size);
        }
        *best_price = match event.side {
            Side::Buy => book.iter().next_back().map(|(&price, &size)| Level::new(price, size)),
            Side::Sell => book.iter().next().map(|(&price, &size)| Level::new(price, size)),
        };
    }

    fn process_trade(&mut self, event: Event<V>) {
        let (book, best_price) = match event.side {
            Side::Buy => (&mut self.bids, &mut self.best_bid),
            Side::Sell => (&mut self.asks, &mut self.best_ask),
        };
        if let Some(size) = book.get_mut(&event.price) {
            if event.size >= *size {
                book.remove(&event.price);
            } else {
                *size -= event.size;
            }
        }
        *best_price = match event.side {
            Side::Buy => book.iter().next_back().map(|(&price, &size)| Level::new(price, size)),
            Side::Sell => book.iter().next().map(|(&price, &size)| Level::new(price, size)),
        };
    }

    fn process_bbo(&mut self, event: Event<V>) {
        let (book, best_price) = match event.side {
            Side::Buy => {
                self.bids.retain(|&price, _| price <= event.price);
                (&mut self.bids, &mut self.best_bid)
            }
            Side::Sell => {
                self.asks.retain(|&price, _| price >= event.price);
                (&mut self.asks, &mut self.best_ask)
            }
        };
        if event.size == V::ZERO {
            book.remove(&event.price);
        } else {
            book.insert(event.price, event.size);
        }
        *best_price = match event.side {
            Side::Buy => book.iter().next_back().map(|(&price, &size)| Level::new(price, size)),
            Side::Sell => book.iter().next().map(|(&price, &size)| Level::new(price, size)),
        };
    }
}
