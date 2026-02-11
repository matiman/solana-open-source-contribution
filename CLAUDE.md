# Rules

## Build & Test Commands
- `cargo build --release` — always build in release mode for performance work
- `cargo run --release` — run the in-process benchmark (400M orders)
- `cargo run --release --bin gateway` — start the gateway server (default: port 9000, 4 clients)
- `cargo run --release --bin gateway -- <port> <num_clients>` — gateway with custom port/clients
- `cargo run --release --bin client -- <host:port> <order_count>` — start a client (sends mixed instruments)
- `cargo test` — run all tests
- `cargo bench` — run criterion benchmarks

## Code Conventions
- Use `u32` for IDs, prices, quantities; `u64` for timestamps; `u8` for instrument IDs
- Keep `Order` struct at 24 bytes — verify with `std::mem::size_of::<Order>()` test if fields change
- Use `#[repr(C)]` on performance-critical structs to control layout
- Use `#[inline(always)]` on hot-path helpers (e.g., `price_index`)
- Caller-owned buffer pattern for trades: pass `&mut Vec<(Order, Order)>`, caller calls `.clear()`
- No async — sync threads only for performance
- Use `crossbeam-channel` (not `std::sync::mpsc`) for inter-thread communication
- Use `SmallRng` (not `StdRng`) for random generation

## Architecture Rules
- Two modes: in-process benchmark (`main.rs`) and distributed (`gateway` + `client` binaries)
- Each instrument runs in its own matcher thread
- Order book uses fixed arrays indexed by `price - MIN_TICK`, not HashMap
- Price range is 10,000–20,000 ticks. Circuit breaker rejects out-of-range prices
- Batch size is 1024 orders (fits L1 cache) — don't change without benchmarking
- Channel buffer is 100 batches
- Gateway is the trust boundary: serde+bincode on the wire (untrusted), crossbeam channels internally (trusted)
- Gateway validates orders (zero qty, zero price, invalid instrument) before routing to matchers
- Matchers do exchange-level validation only (circuit breaker)

## Performance
- Current baseline: ~160M orders/sec (400M orders in <2.5s on 16 cores)
- Always benchmark before and after changes with `cargo run --release`
- Generator (timestamping) is the bottleneck, not the matcher
