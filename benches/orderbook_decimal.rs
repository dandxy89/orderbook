#![allow(clippy::unit_arg)]

use std::f64::consts::PI;

use divan::{black_box, Bencher};
use freya_ob::{
    books::{array_orderbook::ArrayOrderbook, btree_orderbook::BTreeOrderBook, interface::OrderBook},
    event::Event,
    event_kind::EventKind,
    side::Side,
};
use rand::{distributions::Uniform, prelude::Distribution as _, rngs::StdRng, Rng as _, SeedableRng as _};
use rust_decimal::{prelude::FromPrimitive as _, Decimal};

const PRICE_LEVELS: usize = 100;
const MAX_SIZE: f64 = 100.0;

fn main() {
    divan::main();
}

fn setup<T: OrderBook<Decimal> + Default>(skip: bool) -> (T, Vec<Event<Decimal>>) {
    let mut ob = T::default();
    for i in 0..500 {
        let price = Decimal::from_f64(1000.0 + (i as f64 * PI / 2.0).sin() * 10.0).unwrap();
        let size = Decimal::from_f64(100.0 + (i as f64 * PI / 4.0).sin() * 50.0).unwrap();
        ob.process(Event::new(EventKind::L2, Side::Buy, price - Decimal::from(5), size, 0));
        ob.process(Event::new(EventKind::L2, Side::Sell, price + Decimal::from(5), size, 0));
    }

    let records = if !skip {
        let event_dist = Uniform::new(0, 100);
        let price_dist = Uniform::new(0, PRICE_LEVELS);
        let size_dist = Uniform::new(0.0, MAX_SIZE);
        let side_dist = Uniform::new(0, 2);
        let mut rng = StdRng::from_seed([42; 32]);

        let mut records = Vec::with_capacity(100_000);
        for i in 0..100_000 {
            let side = if side_dist.sample(&mut rng) == 0 { Side::Buy } else { Side::Sell };
            let price_level = price_dist.sample(&mut rng) as f64;
            let price_offset = if side == Side::Buy { -0.5 } else { 0.5 };
            let price = 1000.0 + price_level * 0.1 + price_offset;
            let size = if rng.gen_bool(0.1) { 0.0 } else { size_dist.sample(&mut rng) };

            let price = Decimal::from_f64(price).unwrap();
            let size = Decimal::from_f64(size).unwrap();

            let event_type = event_dist.sample(&mut rng);
            let event_kind = match event_type {
                0..=69 => EventKind::L2,     // 70% L2 updates
                70..=89 => EventKind::Trade, // 20% trades
                _ => EventKind::BBO,         // 10% BBO updates
            };
            records.push(Event::new(event_kind, side, price, size, i as i64));
        }

        records
    } else {
        vec![]
    };

    (ob, records)
}

fn generate_price_size(i: usize) -> (Decimal, Decimal) {
    let price = Decimal::from_f64(1000.0 + (i as f64 * PI / 2.0).sin() * 10.0).unwrap();
    let size = Decimal::from_f64(100.0 + (i as f64 * PI / 4.0).sin() * 50.0).unwrap();
    (price, size)
}

#[divan::bench(name = "l2_updates/array")]
fn bench_array_l2_updates(bencher: Bencher) {
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(true)).bench_refs(|(ob, _)| {
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
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(true)).bench_refs(|(ob, _)| {
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
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(true)).bench_refs(|(ob, _)| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::Trade,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size / Decimal::from(2),
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "trades/btree")]
fn bench_btree_trades(bencher: Bencher) {
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(true)).bench_refs(|(ob, _)| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::Trade,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                size / Decimal::from(2),
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "mixed_updates/array")]
fn bench_array_mixed_updates(bencher: Bencher) {
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(true)).bench_refs(|(ob, _)| {
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
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(true)).bench_refs(|(ob, _)| {
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
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(true)).bench_refs(|(ob, _)| {
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
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(true)).bench_refs(|(ob, _)| {
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
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(true)).bench_refs(|(ob, _)| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::L2,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                if i % 3 == 0 { Decimal::ZERO } else { size },
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "rapid_updates/btree")]
fn bench_btree_rapid_updates(bencher: Bencher) {
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(true)).bench_refs(|(ob, _)| {
        for i in 0..10_000 {
            let (price, size) = generate_price_size(i);
            black_box(ob.process(Event::new(
                EventKind::L2,
                if i % 2 == 0 { Side::Buy } else { Side::Sell },
                price,
                if i % 3 == 0 { Decimal::ZERO } else { size },
                i as i64,
            )));
        }
    });
}

#[divan::bench(name = "depth_maintenance/array")]
fn bench_array_depth_maintenance(bencher: Bencher) {
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(true)).bench_refs(|(ob, _)| {
        for i in 0..10_000 {
            let base_price = 1000.0 + (i as f64 * PI / 8.0).sin() * 50.0;
            // Add multiple levels
            for j in 0..5 {
                let price = Decimal::from_f64(base_price + j as f64).unwrap();
                let size = Decimal::from_f64(100.0 + (j as f64 * PI / 4.0).sin() * 20.0).unwrap();
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
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(true)).bench_refs(|(ob, _)| {
        for i in 0..10_000 {
            let base_price = 1000.0 + (i as f64 * PI / 8.0).sin() * 50.0;
            // Add multiple levels
            for j in 0..5 {
                let price = Decimal::from_f64(base_price + j as f64).unwrap();
                let size = Decimal::from_f64(100.0 + (j as f64 * PI / 4.0).sin() * 20.0).unwrap();
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

#[divan::bench(name = "random/btree")]
fn bench_btree_random(bencher: Bencher) {
    bencher.with_inputs(|| setup::<BTreeOrderBook<Decimal>>(false)).bench_values(|(mut ob, records)| {
        for event in records {
            black_box(ob.process(event));
        }
    });
}

#[divan::bench(name = "random/array")]
fn bench_array_random(bencher: Bencher) {
    bencher.with_inputs(|| setup::<ArrayOrderbook<300, Decimal>>(false)).bench_values(|(mut ob, records)| {
        for event in records {
            black_box(ob.process(event));
        }
    });
}
