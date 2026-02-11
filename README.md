# Order Book Matching Engine

A high-performance limit order matching engine built in Rust. Processes 400M orders across 4 instruments at ~160M orders/sec on 16 cores. i.e. ~2.5 sec for 400M orders.

## Features

- **Multi-Instrument**: 4 trading pairs (BTCUSDC, SOLUSDC, ETHUSDC, HYPEUSDC), each on its own matcher thread
- **Distributed Architecture**: Separate client and gateway server processes communicating over TCP
- **Gateway Validation**: Trust boundary — validates and routes orders by instrument before they reach the matcher
- **Limit Order Book (LOB)**: Array-indexed price levels with FIFO queues (zero hashing overhead)
- **Integer Tick Prices**: All prices in ticks (1 tick = $0.01) — no floats, just like real exchanges
- **Partial Fills**: Orders match across multiple counterparties
- **Batch Processing**: Orders sent in batches of 1024 (fits L1 cache) for throughput
- **Smart Constructor**: Invalid orders (zero qty, zero price) rejected at creation
- **Circuit Breaker**: Orders outside the instrument's tick range are rejected
- **Defense in Depth**: Validation at client boundary (gateway) and exchange level (matcher)
- **Compact Order Struct**: 24 bytes with `#[repr(C)]` layout (down from 40 bytes)
- **serde + bincode**: Safe wire protocol for untrusted client data (no raw memory casting)

## How to Run

### In-Process Benchmark (single process, 8 threads)
```bash
cargo run --release              # 400M orders (100M per instrument)
cargo test                       # Run all tests
cargo bench                      # Run benchmarks
```

### Distributed Mode (gateway + clients over TCP)
```bash
# Terminal 1: Start gateway server
cargo run --release --bin gateway

# Terminals 2-5: Start 4 clients (each sends 100M mixed-instrument orders)
cargo run --release --bin client -- 127.0.0.1:9000 100000000
cargo run --release --bin client -- 127.0.0.1:9000 100000000
cargo run --release --bin client -- 127.0.0.1:9000 100000000
cargo run --release --bin client -- 127.0.0.1:9000 100000000
```

## Architecture

### In-Process Mode
```
Instrument 0 (BTCUSDC):  Generator Thread ──► bounded(100) channel ──► Matching Engine Thread
Instrument 1 (SOLUSDC):  Generator Thread ──► bounded(100) channel ──► Matching Engine Thread
Instrument 2 (ETHUSDC):  Generator Thread ──► bounded(100) channel ──► Matching Engine Thread
Instrument 3 (HYPEUSDC): Generator Thread ──► bounded(100) channel ──► Matching Engine Thread
```

### Distributed Mode
```
  UNTRUSTED                    TRUST BOUNDARY                    TRUSTED
┌──────────┐                  ┌──────────────┐               ┌──────────────┐
│ Client 0  │── TCP/serde ──►│              │               │ Matcher BTC  │
│ Client 1  │── TCP/serde ──►│   Gateway    │── crossbeam ─►│ Matcher SOL  │
│ Client 2  │── TCP/serde ──►│  (validate,  │   channels    │ Matcher ETH  │
│ Client 3  │── TCP/serde ──►│   route)     │               │ Matcher HYPE │
└──────────┘                  └──────────────┘               └──────────────┘
```

Each client generates orders for all 4 instruments randomly (clients don't know about routing). The gateway deserializes, validates (zero qty, zero price, invalid instrument), then routes valid orders by instrument to the correct matcher thread via bounded crossbeam channels.

Thread count: 4 client processes (1 thread each) + 1 gateway process (1 main + 4 handlers + 4 matchers) = 13 threads on 16 cores.

## Project Structure

- `order.rs` — Order struct (24 bytes, `#[repr(C)]`) with serde derives, validation, and smart constructor
- `order_book.rs` — LimitOrderBook with array-indexed price levels (true O(1) lookup, no hashing)
- `matching_engine.rs` — MatchingEngine that owns the order book, consumes order batches, tracks stats
- `generator.rs` — Order generator with batch sending (1024 orders/batch, ~1% invalid orders for testing)
- `instrument.rs` — Instrument definitions (BTCUSDC, SOLUSDC, ETHUSDC, HYPEUSDC)
- `protocol.rs` — Wire protocol: length-prefixed bincode-encoded batches over TCP
- `main.rs` — In-process benchmark: spawns thread pairs per instrument
- `bin/gateway.rs` — Gateway server: accepts TCP clients, validates, routes to matchers
- `bin/client.rs` — Client process: generates orders, sends over TCP to gateway

## Performance Journey

| Milestone | Orders | Throughput | Key Change |
|-----------|--------|------------|------------|
| Task 8 baseline | 1M | ~3.6M orders/sec | Initial implementation |
| HashMap → fixed array + async → sync | 100M | ~25M orders/sec | Eliminated hashing and runtime overhead |
| 4 instruments + batching + struct padding | 400M | ~160M orders/sec | Parallelism + cache efficiency |
| Distributed (gateway + TCP clients) | 400K | TBD | Network boundary with serde validation |
