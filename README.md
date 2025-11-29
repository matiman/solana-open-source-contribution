# Solana Open Source Contribution - Order Book Matching Engine

A high-performance order book matching engine built in Rust using async/await with Tokio. The system generates orders concurrently and matches buy/sell orders at the same price level, processing millions of orders efficiently.

## MVP Features

- **Concurrent Processing**: Generator and matcher run in parallel using async channels
- **Order Matching**: Matches buy and sell orders at the same price (FIFO)
- **High Throughput**: Processes 1M+ orders with performance statistics
- **Async Architecture**: Built with Tokio for efficient async I/O

## How to Run

### Prerequisites

- Rust (latest stable version)
- Cargo

### Build and Run

```bash
cd order_book
cargo run --release
```

The program will:

1. Generate 1,000,000 random orders (buy/sell at price levels 100-104)
2. Match orders concurrently as they're generated
3. Display statistics including matched orders, queued orders, and throughput

### Run Tests

```bash
cd order_book
cargo test
```

## Project Structure

- `order.rs` - Order data structure and Side enum
- `order_book.rs` - Order book implementation with price-level queues
- `generator.rs` - Async order generator
- `matcher.rs` - Order matching engine
- `main.rs` - Main entry point with concurrent task orchestration

## Note

- Uncomment Println! statement in the matcher to see print statements but it affects performance.
