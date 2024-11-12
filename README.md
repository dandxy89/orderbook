# Crypto Orderbook Implementation

This is my attempt at implementing an Orderbook for managing a Local Limit Order Book.

All implementations are capable of handling; L2 Updates, Best Bid and Best Ask instants, and Trades.

## Orderbook Implementation

This project includes two implementations of an Orderbook:

1. Array-based Orderbook
2. BTree-based Orderbook

## Custom Decimal Type

This project also include `FixedDecimal` which will replace `rust_decimal::Decimal`. Requires work to replace this!


## Benchmarks

```bash
     Running benches/orderbook.rs (target/release/deps/orderbook-7f1b836ae690cb2c)
Timer precision: 38 ns
orderbook                   fastest       │ slowest       │ median        │ mean          │ samples │ iters
├─ depth_maintenance/array  15.39 ms      │ 19.89 ms      │ 15.8 ms       │ 15.93 ms      │ 100     │ 100
├─ depth_maintenance/btree  20.2 ms       │ 32.01 ms      │ 20.64 ms      │ 21.47 ms      │ 100     │ 100
├─ l2_updates/array         5.071 ms      │ 12.59 ms      │ 5.179 ms      │ 5.977 ms      │ 100     │ 100
├─ l2_updates/btree         4.957 ms      │ 8.44 ms       │ 5.241 ms      │ 5.448 ms      │ 100     │ 100
├─ mixed_updates/array      5.126 ms      │ 8.331 ms      │ 5.469 ms      │ 5.646 ms      │ 100     │ 100
├─ mixed_updates/btree      5.606 ms      │ 9.488 ms      │ 5.658 ms      │ 6.011 ms      │ 100     │ 100
├─ rapid_updates/array      5.193 ms      │ 9.912 ms      │ 5.251 ms      │ 5.683 ms      │ 100     │ 100
├─ rapid_updates/btree      5.291 ms      │ 6.9 ms        │ 5.338 ms      │ 5.486 ms      │ 100     │ 100
├─ snapshot_updates/array   5.12 ms       │ 6.995 ms      │ 5.193 ms      │ 5.363 ms      │ 100     │ 100
├─ snapshot_updates/btree   5.702 ms      │ 9.294 ms      │ 6.019 ms      │ 6.398 ms      │ 100     │ 100
├─ trades/array             5.057 ms      │ 8.483 ms      │ 5.199 ms      │ 5.471 ms      │ 100     │ 100
╰─ trades/btree             5.185 ms      │ 8.316 ms      │ 5.333 ms      │ 5.51 ms       │ 100     │ 100
```

