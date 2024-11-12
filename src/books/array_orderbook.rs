use std::{
    iter::Sum,
    ops::{Add, Div, Mul, Sub},
};

use crate::{
    books::interface::OrderBook,
    buffers::buffer::Buffer,
    decimals::decimal_type::DecimalType,
    event::Event,
    event_kind::EventKind,
    level::Level,
    metrics::{MetricsCalculator, OrderbookMetrics},
    side::Side,
};

#[derive(Debug)]
/// Here is a brief explanation of each field:
///
/// - `Best_bid`: Stores the best bid price (i.e., the highest price at which someone is willing to buy).
/// - `Best_ask`: Stores the best ask price (i.e., the lowest price at which someone is willing to sell).
/// - `Bids`: A buffer of size N that stores bid levels (i.e., prices and quantities at which people are willing to buy).
/// - `Asks`: A buffer of size N that stores ask levels (i.e., prices and quantities at which people are willing to sell).
/// - `Ts`: Stores the timestamp of the last update.
/// - `Sequence_id`: Stores the sequence ID of the last update.
/// - `Has_moved`: A boolean flag indicating whether the order book has moved since the last update.
///
pub struct ArrayOrderbook<const N: usize, V>
where
    V: DecimalType + PartialOrd,
{
    pub best_bid: Option<Level<V>>,
    pub best_ask: Option<Level<V>>,
    pub bids: Buffer<N, V>,
    pub asks: Buffer<N, V>,
    pub ts: i64,
    pub sequence_id: u64,
    pub has_moved: bool,
}

impl<const N: usize, V> MetricsCalculator<V> for ArrayOrderbook<N, V>
where
    V: DecimalType + PartialOrd + Sub<Output = V> + Add<Output = V> + Mul<Output = V> + Div<Output = V> + Copy + Ord + Sum,
{
    fn best_bid(&self) -> Option<Level<V>> {
        self.best_bid
    }

    fn best_ask(&self) -> Option<Level<V>> {
        self.best_ask
    }
}

impl<const N: usize, V> OrderBook<V> for ArrayOrderbook<N, V>
where
    V: DecimalType + PartialOrd + Sub<Output = V> + Add<Output = V> + Mul<Output = V> + Div<Output = V> + Copy + Ord + Sum,
{
    #[inline]
    /// Processes an event by updating the internal order book state based on the event kind.
    ///
    /// - If the event is older than the current timestamp (`ts`), it will be ignored.
    /// - Updates the timestamp and handles the sequence ID to ensure the event is processed in the correct order.
    /// - Depending on the event kind:
    ///   - `Trade`: Calls `process_trade` to handle trade events and update bid/ask levels.
    ///   - `Instant`: Calls `process_bbo` to handle Best Bid/Offer events and adjust the order book accordingly.
    ///   - `L2`: Calls `process_lvl2` to handle Level 2 updates and maintain the depth of the order book.
    ///
    fn process(&mut self, event: Event<V>) {
        let ts = event.timestamp;
        // Ignore old events
        if ts < self.ts {
            return;
        }

        // Handle sequence_id (if its non-zero) and timestamp
        if event.sequence_id == 0
            || self.sequence_id == 0
            || event.sequence_id == self.sequence_id
            || event.sequence_id > self.sequence_id
        {
            self.ts = ts;
            if event.sequence_id != 0 {
                self.sequence_id = event.sequence_id;
            }
            match event.kind {
                EventKind::Trade => self.process_trade(event),
                EventKind::BBO => self.process_bbo(event),
                EventKind::L2 => self.process_lvl2(event),
            }
        }
    }

    #[inline]
    fn best_bid(&mut self) -> Option<Level<V>> {
        self.has_moved = false;
        self.best_bid
    }

    #[inline]
    fn best_ask(&mut self) -> Option<Level<V>> {
        self.has_moved = false;
        self.best_ask
    }

    #[inline]
    #[must_use]
    /// Calculate various orderbook metrics up to a specified depth
    ///
    /// Returns a struct containing different market microstructure indicators
    fn calculate_metrics(&self, depth: usize) -> OrderbookMetrics<V> {
        let mut bid_sizes = Vec::with_capacity(depth);
        let mut ask_sizes = Vec::with_capacity(depth);
        let mut bid_prices = Vec::with_capacity(depth);
        let mut ask_prices = Vec::with_capacity(depth);

        // Collect bid and ask data up to specified depth
        for i in 0..depth {
            if i < self.bids.len {
                unsafe {
                    let level = self.bids.get_unchecked(i);
                    bid_sizes.push(level.size);
                    bid_prices.push(level.price);
                }
            }
            if i < self.asks.len {
                unsafe {
                    let level = self.asks.get_unchecked(i);
                    ask_sizes.push(level.size);
                    ask_prices.push(level.price);
                }
            }
        }

        self.calculate_metrics_internal(bid_sizes, ask_sizes, bid_prices, ask_prices)
    }
}

#[cfg(feature = "rust_decimal")]
impl Default for ArrayOrderbook<300, rust_decimal::Decimal> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "fixed_decimal")]
impl Default for ArrayOrderbook<300, crate::decimals::fixed_decimal::FixedDecimal> {
    #[inline]
    #[must_use]
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, V> ArrayOrderbook<N, V>
where
    V: DecimalType + PartialOrd + Copy + Ord,
{
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            best_bid: None,
            best_ask: None,
            bids: Buffer::new(true),
            asks: Buffer::new(false),
            ts: 0,
            sequence_id: 0,
            has_moved: false,
        }
    }

    #[inline(always)]
    fn process_lvl2(&mut self, event: Event<V>) {
        let (buffer, best_price) = match event.side {
            Side::Buy => (&mut self.bids, &mut self.best_bid),
            Side::Sell => (&mut self.asks, &mut self.best_ask),
        };

        // If the size is zero, remove the level
        if event.size == V::ZERO {
            if let Ok(to_remove) = buffer.find_index(event.price, event.side.is_buy()) {
                let removed = buffer.remove(to_remove);
                if let Some(best) = *best_price {
                    if removed == best.price {
                        *best_price = buffer.first();
                        self.has_moved = true;
                    }
                }
            }
            return;
        }

        // If the size is non-zero, insert or modify the level
        match buffer.find_index(event.price, event.side.is_buy()) {
            Ok(to_modify) => {
                buffer.modify(to_modify, event.size);
                if to_modify == 0 {
                    *best_price = buffer.first();
                }
            }
            Err(to_insert) => {
                buffer.insert(to_insert, event.to_level());
                if to_insert == 0 {
                    *best_price = buffer.first();
                }
            }
        }
    }

    #[inline]
    /// Process a trade event. This function is responsible for updating the bid/ask
    /// buffer(s) and best bid/ask price(s) based on the trade event.
    ///
    /// - If the trade event is a buy, it will decrement or remove the best bid level
    /// - if the size of the trade is greater than or equal to the size of the level.
    /// - If the trade event is a sell, it will decrement or remove the best ask level
    /// - if the size of the trade is greater than or equal to the size of the level.
    ///
    /// If the level is removed, the best bid/ask price will be updated to the new
    /// best bid/ask price(s) in the buffer(s).
    fn process_trade(&mut self, event: Event<V>) {
        match event.side {
            Side::Buy => {
                if let Ok(index) = self.bids.find_index(event.price, true) {
                    // SAFETY: index is valid from find_index
                    unsafe {
                        let level = self.bids.get_unchecked_mut(index);
                        if event.size >= level.size {
                            self.bids.remove(index);
                            if index == 0 {
                                self.best_bid = self.bids.first();
                            }
                        } else {
                            self.bids.modify(index, event.size);
                        }
                    }
                    if index == 0 {
                        self.best_bid = self.bids.first();
                    }
                }
            }
            Side::Sell => {
                if let Ok(index) = self.asks.find_index(event.price, false) {
                    // SAFETY: index is valid from find_index
                    unsafe {
                        let level = self.asks.get_unchecked_mut(index);
                        if event.size >= level.size {
                            self.asks.remove(index);
                            if index == 0 {
                                self.best_ask = self.asks.first();
                            }
                        } else {
                            self.asks.modify(index, event.size);
                        }
                    }
                    if index == 0 {
                        self.best_ask = self.asks.first();
                    }
                }
            }
        }
    }

    #[inline]
    /// Process a BBO (Best Bid/Offer) event. This function is responsible for updating the bid/ask
    /// buffer(s) and best bid/ask price(s) based on the BBO event.
    ///
    /// - If the BBO is a buy, it will remove any bid price levels that are better than the new BBO.
    /// - If the BBO is a sell, it will remove any ask price levels that are better than the new BBO.
    /// - If the BBO price level already exists, it will be modified to the new size. If the size is zero,
    ///   the level will be removed.
    /// - If the BBO price level does not exist and the size is greater than zero, the level will be
    ///   inserted into the buffer.
    /// - The best bid/ask price will be updated to the new best bid/ask price(s) in the buffer(s).
    ///
    fn process_bbo(&mut self, event: Event<V>) {
        let (buffer, best_price) =
            if event.side.is_buy() { (&mut self.bids, &mut self.best_bid) } else { (&mut self.asks, &mut self.best_ask) };

        while let Some(best) = buffer.first() {
            if (event.side.is_buy() && best.price > event.price) || (!event.side.is_buy() && best.price < event.price) {
                buffer.remove(0);
            } else {
                break;
            }
        }

        // Handle the BBO price level
        if event.size == V::ZERO {
            if let Ok(index) = buffer.find_index(event.price, event.side.is_buy()) {
                buffer.remove(index);
            }
        } else {
            match buffer.find_index(event.price, event.side.is_buy()) {
                Ok(index) => buffer.modify(index, event.size),
                Err(index) => buffer.insert(index, event.to_level()),
            }
        }

        *best_price = buffer.first();
    }
}

#[cfg(test)]
#[cfg(feature = "rust_decimal")]
mod test {
    use std::f64::consts::PI;

    use rust_decimal::{prelude::FromPrimitive as _, Decimal};
    use rust_decimal_macros::dec;

    use crate::{
        books::{
            array_orderbook::{ArrayOrderbook, Event},
            interface::OrderBook as _,
        },
        event_kind::EventKind,
        side::Side,
    };

    #[test]
    /// Tests the best bid/offer functionality of the orderbook.
    ///
    /// 1. Sets a best bid/offer and verifies the orderbook state.
    /// 2. Updates the best bid to a better price, verifies the orderbook state.
    /// 3. Updates the best bid to the same price, with a lower quantity, verifies the orderbook state.
    /// 4. Updates the best bid to a worse price, verifies the orderbook state.
    fn bbo() {
        let mut lob = ArrayOrderbook::<3, Decimal>::new();
        let bbo_bid = Event::new(EventKind::BBO, Side::Buy, dec!(100.0), dec!(1.), 10001);
        let bbo_ask = Event::new(EventKind::BBO, Side::Sell, dec!(100.1), dec!(1.), 10001);

        lob.process(bbo_ask);
        lob.process(bbo_bid);
        insta::assert_debug_snapshot!(lob);

        let bbo_bid = Event::new(EventKind::BBO, Side::Buy, dec!(100.05), dec!(1.), 10002);
        lob.process(bbo_bid);
        insta::assert_debug_snapshot!(lob);

        let bbo_bid = Event::new(EventKind::BBO, Side::Buy, dec!(100.05), dec!(0.), 10003);
        lob.process(bbo_bid);
        insta::assert_debug_snapshot!(lob);

        let bbo_bid = Event::new(EventKind::BBO, Side::Buy, dec!(100.04), dec!(0.), 10004);
        lob.process(bbo_bid);
        insta::assert_debug_snapshot!(lob);
    }

    #[test]
    /// Test that a trade event is correctly processed. Given a BBO of 100.0 / 100.1, with a quantity
    /// of 2.0 and 1.1 respectively, process a buy trade of 1.0 and a sell trade of 1.0. Verify
    /// that the orderbook is updated to reflect the new quantities.
    fn trade() {
        let mut lob = ArrayOrderbook::<3, Decimal>::new();
        let bbo_bid = Event::new(EventKind::BBO, Side::Buy, dec!(100.0), dec!(2.), 10001);
        let bbo_ask = Event::new(EventKind::BBO, Side::Sell, dec!(100.1), dec!(1.1), 10001);

        lob.process(bbo_ask);
        lob.process(bbo_bid);

        let bid_trade = Event::new(EventKind::Trade, Side::Buy, dec!(100.0), dec!(1.), 10002);
        let ask_trade = Event::new(EventKind::Trade, Side::Sell, dec!(100.1), dec!(1.), 10002);
        lob.process(bid_trade);
        lob.process(ask_trade);
        insta::assert_debug_snapshot!(lob);
    }

    #[test]
    /// Test that the orderbook correctly updates the quantity of an existing level
    /// when given a level 2 update event. Additionally, test that the orderbook
    /// correctly decrements the quantity of a level when given a trade event.
    /// Finally, test that the orderbook correctly handles a level 2 update event
    /// that removes a level.
    fn lv2() {
        let mut lob = ArrayOrderbook::<3, Decimal>::new();
        let event1 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(2.), 10001);
        let event2 = Event::new(EventKind::L2, Side::Buy, dec!(100.1), dec!(1.1), 10001);
        let event3 = Event::new(EventKind::L2, Side::Buy, dec!(100.2), dec!(0.1), 10001);
        let event4 = Event::new(EventKind::Trade, Side::Buy, dec!(100.0), dec!(1.), 10001);

        lob.process(event1);
        lob.process(event2);
        lob.process(event3);
        lob.process(event4);

        insta::assert_debug_snapshot!(lob);

        let bbo_bid = Event::new(EventKind::BBO, Side::Buy, dec!(100.0), dec!(2.), 10002);

        lob.process(bbo_bid);

        insta::assert_debug_snapshot!(lob);
    }

    #[test]
    fn test_orderbook_metrics() {
        let mut lob = ArrayOrderbook::<5, Decimal>::new();
        // Add some sample orders
        let events = vec![
            Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(1.0), 1),
            Event::new(EventKind::L2, Side::Buy, dec!(99.0), dec!(2.0), 1),
            Event::new(EventKind::L2, Side::Sell, dec!(101.0), dec!(1.5), 1),
            Event::new(EventKind::L2, Side::Sell, dec!(102.0), dec!(1.0), 1),
            Event::new(EventKind::L2, Side::Buy, dec!(95.0), dec!(1.0), 1),
            Event::new(EventKind::L2, Side::Buy, dec!(98.0), dec!(2.0), 1),
            Event::new(EventKind::L2, Side::Sell, dec!(102.21), dec!(1.5), 1),
            Event::new(EventKind::L2, Side::Sell, dec!(104.1), dec!(1.0), 1),
        ];
        for event in events {
            lob.process(event);
        }
        insta::assert_debug_snapshot!(lob.calculate_metrics(5));
    }

    fn sin_generation_fn(base_price: f64, i: usize) -> Decimal {
        let value = (i as f64 * PI / 8.0).sin().mul_add(50.0, base_price);
        Decimal::from_f64(value).unwrap()
    }

    #[test]
    fn test_rotating_price() {
        let mut ob: ArrayOrderbook<10, Decimal> = ArrayOrderbook::new();
        let base_price = 1000.0;

        for i in 0..16 {
            // Buy side
            let price = sin_generation_fn(base_price - 1., i);
            let size = sin_generation_fn(100.0, i);
            ob.process(Event::new(EventKind::BBO, Side::Buy, price, size, 0));
            // Sell side
            let size = sin_generation_fn(100.0, i);
            let price = sin_generation_fn(base_price + 1., i);
            ob.process(Event::new(EventKind::BBO, Side::Sell, price, size, 0));
            // Assert
            insta::assert_debug_snapshot!(ob);
        }

        insta::assert_debug_snapshot!(ob.calculate_metrics(10));
    }

    #[test]
    fn test_level_removal() {
        let mut lob = ArrayOrderbook::<5, Decimal>::new();
        // Add levels
        let events = vec![
            Event::new(EventKind::L2, Side::Buy, dec!(100.), dec!(1.), 1),
            Event::new(EventKind::L2, Side::Buy, dec!(99.), dec!(2.), 2),
        ];
        for event in events {
            lob.process(event);
        }
        // Remove first level by setting size to 0
        lob.process(Event::new(EventKind::L2, Side::Buy, dec!(100.), Decimal::ZERO, 2));
        assert_eq!(lob.bids.len, 1);
        unsafe {
            assert_eq!(lob.bids.get_unchecked(0).price, dec!(99.));
        }
    }

    #[test]
    fn test_quote_imbalance() {
        let mut lob = ArrayOrderbook::<5, Decimal>::new();
        // Add equal bid and ask volumes
        lob.process(Event::new(EventKind::L2, Side::Buy, dec!(100.), dec!(1.), 1));
        lob.process(Event::new(EventKind::L2, Side::Sell, dec!(101.), dec!(1.), 2));
        let metrics = lob.calculate_metrics(5);
        insta::assert_debug_snapshot!(metrics);

        // Add more bid volume
        lob.process(Event::new(EventKind::L2, Side::Buy, dec!(99.), dec!(2.), 3));
        let metrics = lob.calculate_metrics(5);
        insta::assert_debug_snapshot!(metrics);
    }
}

#[cfg(test)]
#[cfg(feature = "rust_decimal")]
mod sequence_tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    use crate::{
        books::{array_orderbook::ArrayOrderbook, interface::OrderBook as _},
        event::Event,
        event_kind::EventKind,
        side::Side,
    };

    #[test]
    /// Test that events with sequence IDs are processed in order and out-of-order
    /// events are ignored
    fn test_sequence_order() {
        let mut ob = ArrayOrderbook::<5, Decimal>::new();
        // Process initial state
        let event1 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(1.0), 0).with_sequence_id(1);
        ob.process(event1);
        assert_eq!(ob.sequence_id, 1);
        // Process event with higher sequence - should update
        let event2 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(2.0), 1).with_sequence_id(2);
        ob.process(event2);
        assert_eq!(ob.sequence_id, 2);
        assert_eq!(ob.best_bid().unwrap().size, dec!(2.0));
        // Process older event - should be ignored
        let old_event = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(0.5), 2).with_sequence_id(1);
        ob.process(old_event);
        assert_eq!(ob.sequence_id, 2);
        assert_eq!(ob.best_bid().unwrap().size, dec!(2.0));
    }

    #[test]
    /// Test that events with sequence ID zero are always processed
    fn test_zero_sequence() {
        let mut ob = ArrayOrderbook::<5, Decimal>::new();
        // Set initial state with non-zero sequence
        let event1 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(1.0), 0).with_sequence_id(5);
        ob.process(event1);
        assert_eq!(ob.sequence_id, 5);
        // Event with sequence_id 0 should still be processed
        let event2 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(2.0), 1).with_sequence_id(0);
        ob.process(event2);
        assert_eq!(ob.sequence_id, 5); // Sequence ID shouldn't change
        assert_eq!(ob.best_bid().unwrap().size, dec!(2.0)); // But state should update
    }

    #[test]
    /// Test that the orderbook can handle sequence resets (gaps in sequence)
    fn test_sequence_reset() {
        let mut ob = ArrayOrderbook::<5, Decimal>::new();
        // Initial state
        let event1 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(1.0), 0).with_sequence_id(1);
        ob.process(event1);
        // Jump to much higher sequence (simulating reset/reconnect)
        let event2 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(2.0), 1).with_sequence_id(1000);
        ob.process(event2);
        assert_eq!(ob.sequence_id, 1000);
        assert_eq!(ob.best_bid().unwrap().size, dec!(2.0));
        // Ensure we continue processing higher sequences
        let event3 = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(3.0), 2).with_sequence_id(1001);
        ob.process(event3);
        assert_eq!(ob.sequence_id, 1001);
        assert_eq!(ob.best_bid().unwrap().size, dec!(3.0));
    }

    #[test]
    /// Test handling of multiple event types with sequence IDs
    fn test_mixed_event_types() {
        let mut ob = ArrayOrderbook::<5, Decimal>::new();
        // Set up initial state with L2 update
        let l2_event = Event::new(EventKind::L2, Side::Buy, dec!(100.0), dec!(2.0), 0).with_sequence_id(1);
        ob.process(l2_event);
        // Process a trade
        let trade = Event::new(EventKind::Trade, Side::Buy, dec!(100.0), dec!(1.0), 1).with_sequence_id(2);
        ob.process(trade);
        assert_eq!(ob.best_bid().unwrap().size, dec!(1.0));
        // Process a BBO update
        let bbo = Event::new(EventKind::BBO, Side::Buy, dec!(101.0), dec!(1.5), 2).with_sequence_id(3);
        ob.process(bbo);
        assert_eq!(ob.best_bid().unwrap().price, dec!(101.0));
        assert_eq!(ob.best_bid().unwrap().size, dec!(1.5));
        // Try to process old events of each type - should all be ignored
        let old_l2 = Event::new(EventKind::L2, Side::Buy, dec!(99.0), dec!(1.0), 0).with_sequence_id(2);
        let old_trade = Event::new(EventKind::Trade, Side::Buy, dec!(101.0), dec!(0.5), 0).with_sequence_id(2);
        let old_bbo = Event::new(EventKind::BBO, Side::Buy, dec!(102.0), dec!(2.0), 0).with_sequence_id(1);
        // Process old events
        ob.process(old_l2);
        ob.process(old_trade);
        ob.process(old_bbo);
        // State should remain unchanged
        assert_eq!(ob.sequence_id, 3);
        assert_eq!(ob.best_bid().unwrap().price, dec!(101.0));
        assert_eq!(ob.best_bid().unwrap().size, dec!(1.5));
    }
}
