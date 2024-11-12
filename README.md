# Rust Orderbook Implementation

This is my attempt at implementing an Orderbook for managing a Local Limit Order Book.

All implementations are capable of handling; L2 Updates, Best Bid and Best Ask instants, and Trades.

The goal is to attempt to understand what could make an Orderbook implementation fast so will include:

- Custom DataType to manage working with f64, Decimal Types and my own Custom Types
- Does SIMD binary search and another tricks make much difference
- `ReversedVec` vs `Buffer` do they make a difference

## To the reader of this README

Worth reading this <https://en.algorithmica.org/hpc/>

## Implementation Overview

This project includes two implementations of an Orderbook so far:

1. Array-based Orderbook
2. BTree-based Orderbook

and two implementations of Array buffering:

1. `Buffer`
2. `ReveredVec` - should, in theory, be faster

and the intention is to support the following tasks:

```rust
pub enum EventKind {
    /// Trade events
    Trade,
    /// Best Bid/Offer events
    BBO,
    /// Level 2 events (prices and sizes)
    L2,
}
```

## Custom Decimal Type

This project also include `FixedDecimal` which could be used to replace `rust_decimal::Decimal`.

Also, with a little work, supports `serde`.

## Benchmarks

Using `rust_decimal::Decimal`:

```bash
Timer precision: 38 ns
orderbook_decimal           fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ depth_maintenance/array  15.55 ms      │ 24.92 ms      │ 15.95 ms      │ 16.37 ms      │ 100     │ 100
├─ depth_maintenance/btree  20.27 ms      │ 81.54 ms      │ 20.49 ms      │ 21.99 ms      │ 100     │ 100
├─ l2_updates/array         5.096 ms      │ 55.63 ms      │ 5.226 ms      │ 6.477 ms      │ 100     │ 100
├─ l2_updates/btree         4.948 ms      │ 7.851 ms      │ 4.973 ms      │ 5.059 ms      │ 100     │ 100
├─ mixed_updates/array      5.119 ms      │ 7.597 ms      │ 5.163 ms      │ 5.322 ms      │ 100     │ 100
├─ mixed_updates/btree      5.628 ms      │ 7.637 ms      │ 5.673 ms      │ 5.841 ms      │ 100     │ 100
├─ rapid_updates/array      5.233 ms      │ 7.798 ms      │ 5.286 ms      │ 5.546 ms      │ 100     │ 100
├─ rapid_updates/btree      5.29 ms       │ 6.818 ms      │ 5.318 ms      │ 5.427 ms      │ 100     │ 100
├─ snapshot_updates/array   5.118 ms      │ 6.531 ms      │ 5.14 ms       │ 5.247 ms      │ 100     │ 100
├─ snapshot_updates/btree   5.77 ms       │ 8.682 ms      │ 5.82 ms       │ 5.966 ms      │ 100     │ 100
├─ trades/array             5.056 ms      │ 10.08 ms      │ 5.098 ms      │ 5.267 ms      │ 100     │ 100
╰─ trades/btree             5.194 ms      │ 7.861 ms      │ 5.231 ms      │ 5.352 ms      │ 100     │ 100
```

Using custom `FixedDecimal`:

```bash
     Running benches/orderbook_fixed_decimal.rs (target/release/deps/orderbook_fixed_decimal-898d9311a58564a5)
Timer precision: 38 ns
orderbook_fixed_decimal     fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ depth_maintenance/array  1.562 ms      │ 2.676 ms      │ 1.579 ms      │ 1.617 ms      │ 100     │ 100
├─ depth_maintenance/btree  3.18 ms       │ 5.945 ms      │ 3.215 ms      │ 3.305 ms      │ 100     │ 100
├─ l2_updates/array         489.2 µs      │ 3.986 ms      │ 496.1 µs      │ 625.7 µs      │ 100     │ 100
├─ l2_updates/btree         529.7 µs      │ 2.821 ms      │ 543.5 µs      │ 625 µs        │ 100     │ 100
├─ mixed_updates/array      477.3 µs      │ 1.156 ms      │ 484.4 µs      │ 521.6 µs      │ 100     │ 100
├─ mixed_updates/btree      774.7 µs      │ 4.511 ms      │ 785.7 µs      │ 879.8 µs      │ 100     │ 100
├─ rapid_updates/array      558.1 µs      │ 1.841 ms      │ 573.1 µs      │ 651.6 µs      │ 100     │ 100
├─ rapid_updates/btree      758.6 µs      │ 3.389 ms      │ 765.5 µs      │ 835.9 µs      │ 100     │ 100
├─ snapshot_updates/array   486.8 µs      │ 3.158 ms      │ 496.3 µs      │ 559.3 µs      │ 100     │ 100
├─ snapshot_updates/btree   803.2 µs      │ 2.417 ms      │ 815.7 µs      │ 887.2 µs      │ 100     │ 100
├─ trades/array             368.9 µs      │ 4.035 ms      │ 371 µs        │ 432.3 µs      │ 100     │ 100
╰─ trades/btree             462.3 µs      │ 1.376 ms      │ 462.8 µs      │ 510.5 µs      │ 100     │ 100
```
