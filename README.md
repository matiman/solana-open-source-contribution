# Order Book Matching Engine

A high-performance limit order matching engine built in Rust. Processes 100M orders at ~22M orders/sec using lock-free crossbeam channels and sync threads.

## Features

- **Limit Order Book (LOB)**: Array-indexed price levels with FIFO queues (zero hashing overhead)
- **Integer Tick Prices**: All prices in ticks (1 tick = $0.01) — no floats, just like real exchanges
- **Partial Fills**: Orders match across multiple counterparties
- **Smart Constructor**: Invalid orders (zero qty, zero price) rejected at creation
- **Circuit Breaker**: Orders outside the instrument's tick range are rejected
- **Defense in Depth**: Validation at both order creation and matching engine layers
- **High Throughput**: ~22M orders/sec with ~1% invalid order rejection

## How to Run

```bash
cargo run --release    # Run with 100M orders
cargo test             # Run all tests
cargo bench            # Run benchmarks
```

## Architecture

```
Generator Thread ──► bounded(10,000) channel ──► Matching Engine Thread
   (producer)           (backpressure)               (consumer)
```

 Two threads communicate via a crossbeam bounded channel. The buffer size (10,000) acts as a backpressure mechanism — if the engine can't keep up, the generator blocks on `send()` rather than buffering millions of orders in memory. This mirrors how production exchanges use bounded queues for flow control and deterministic memory usage.

## Project Structure

- `order.rs` - Order struct with integer tick prices and smart constructor validation
- `order_book.rs` - LimitOrderBook with array-indexed price levels (true O(1) lookup, no hashing)
- `matching_engine.rs` - MatchingEngine that owns the order book and processes limit orders
- `generator.rs` - Order generator (~1% invalid orders for testing)
- `main.rs` - Entry point with threaded generator + matching engine
