#![allow(clippy::unit_arg)]

use crypto_orderbook::{
    books::{array_orderbook::ArrayOrderbook, btree_orderbook::BTreeOrderBook},
    decimals::fixed_decimal::FixedDecimal,
    event::Event,
    event_kind::EventKind,
    books::interface::OrderBook,
    side::Side,
};
use divan::{black_box, Bencher};
use std::f64::consts::PI;

fn main() {
    divan::main();
}

fn setup<T: OrderBook<FixedDecimal> + Default>() -> T {
    let mut ob = T::default();
    for i in 0..500 {
        let price = FixedDecimal::from_f64(1000.0 + (i as f64 * PI / 2.0).sin() * 10.0);
        let size = FixedDecimal::from_f64(100.0 + (i as f64 * PI / 4.0).sin() * 50.0);
        ob.process(Event::new(EventKind::L2, Side::Buy, price - FixedDecimal::from_int(5), size, 0));
        ob.process(Event::new(EventKind::L2, Side::Sell, price + FixedDecimal::from_int(5), size, 0));
    }

    ob
}

fn generate_price_size(i: usize) -> (FixedDecimal, FixedDecimal) {
    let price = FixedDecimal::from_f64(1000.0 + (i as f64 * PI / 2.0).sin() * 10.0);
    let size = FixedDecimal::from_f64(100.0 + (i as f64 * PI / 4.0).sin() * 50.0);
    (price, size)
}

#[divan::bench(name = "l2_updates/array")]
fn bench_array_l2_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<ArrayOrderbook<300, FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::L2,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size,
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "l2_updates/btree")]
fn bench_btree_l2_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<BTreeOrderBook<FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::L2,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size,
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "trades/array")]
fn bench_array_trades(bencher: Bencher) {
    bencher.with_inputs(setup::<ArrayOrderbook<300, FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::Trade,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size / FixedDecimal::from_int(2),
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "trades/btree")]
fn bench_btree_trades(bencher: Bencher) {
    bencher.with_inputs(setup::<BTreeOrderBook<FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::Trade,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size / FixedDecimal::from_int(2),
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "mixed_updates/array")]
fn bench_array_mixed_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<ArrayOrderbook<300, FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            let kind = match i % 3 {
                0 => EventKind::L2,
                1 => EventKind::Trade,
                _ => EventKind::BBO,
            };
            black_box(ob.process(Event::new(kind, if i % 2 == 0 { Side::Buy } else { Side::Sell }, price, size, i as i64)));
        }
    });
}

#[divan::bench(name = "mixed_updates/btree")]
fn bench_btree_mixed_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<BTreeOrderBook<FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            let kind = match i % 3 {
                0 => EventKind::L2,
                1 => EventKind::Trade,
                _ => EventKind::BBO,
            };
            black_box(ob.process(Event::new(kind, if i % 2 == 0 { Side::Buy } else { Side::Sell }, price, size, i as i64)));
        }
    });
}

#[divan::bench(name = "snapshot_updates/array")]
fn bench_array_snapshot_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<ArrayOrderbook<300, FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::BBO,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size,
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "snapshot_updates/btree")]
fn bench_btree_snapshot_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<BTreeOrderBook<FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::BBO,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size,
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "rapid_updates/array")]
fn bench_array_rapid_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<ArrayOrderbook<300, FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::L2,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                if i % 3 == 0 { FixedDecimal::ZERO } else { size },
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "rapid_updates/btree")]
fn bench_btree_rapid_updates(bencher: Bencher) {
    bencher.with_inputs(setup::<BTreeOrderBook<FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::L2,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                if i % 3 == 0 { FixedDecimal::ZERO } else { size },
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "depth_maintenance/array")]
fn bench_array_depth_maintenance(bencher: Bencher) {
    bencher.with_inputs(setup::<ArrayOrderbook<300, FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let base_price = 1000.0 + (i as f64 * PI / 8.0).sin() * 50.0;
            // Add multiple levels
            for j in 0..5 {
                let price = FixedDecimal::from_f64(base_price + j as f64);
                let size = FixedDecimal::from_f64(100.0 + (j as f64 * PI / 4.0).sin() * 20.0);
                black_box(ob.process(Event::new(
                    EventKind::L2,
                    if i % 2 == 0 { Side::Buy } else { Side::Sell },
                    price,
                    size,
                    i as i64,
                )));
            }
        }
    });
}

#[divan::bench(name = "depth_maintenance/btree")]
fn bench_btree_depth_maintenance(bencher: Bencher) {
    bencher.with_inputs(setup::<BTreeOrderBook<FixedDecimal>>).bench_refs(|ob| {
        for i in 0..10_000 {
            let base_price = 1000.0 + (i as f64 * PI / 8.0).sin() * 50.0;
            // Add multiple levels
            for j in 0..5 {
                let price = FixedDecimal::from_f64(base_price + j as f64);
                let size = FixedDecimal::from_f64(100.0 + (j as f64 * PI / 4.0).sin() * 20.0);
                black_box(ob.process(Event::new(
                    EventKind::L2,
                    if i % 2 == 0 { Side::Buy } else { Side::Sell },
                    price,
                    size,
                    i as i64,
                )));
            }
        }
    });
}
