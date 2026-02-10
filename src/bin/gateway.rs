use order_book::{
    generator::OrderBatch,
    instrument::{Instrument, ALL_INSTRUMENTS, NUM_INSTRUMENTS},
    matching_engine::MatchingEngine,
    protocol,
};
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Instant;

const DEFAULT_PORT: u16 = 9000;
const DEFAULT_CLIENTS: usize = 4;
const CHANNEL_BUFFER_SIZE: usize = 100;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let port: u16 = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(DEFAULT_PORT);
    let expected_clients: usize = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_CLIENTS);

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).expect("failed to bind");

    let instrument_names: Vec<_> = ALL_INSTRUMENTS
        .iter()
        .map(|id| format!("{}", id))
        .collect();

    println!("Order Book Gateway Server");
    println!("=========================");
    println!("Listening on {}", addr);
    println!("Instruments: {} ({})", NUM_INSTRUMENTS, instrument_names.join(", "));
    println!("Expecting {} client(s)", expected_clients);
    println!();

    // Create per-instrument channels (gateway → matcher)
    let mut senders: Vec<crossbeam_channel::Sender<OrderBatch>> = Vec::with_capacity(NUM_INSTRUMENTS);
    let mut matcher_handles = Vec::with_capacity(NUM_INSTRUMENTS);

    for &instrument in &ALL_INSTRUMENTS {
        let (tx, rx) = crossbeam_channel::bounded(CHANNEL_BUFFER_SIZE);
        senders.push(tx);

        let handle = thread::spawn(move || {
            let start = Instant::now();
            let stats = MatchingEngine::run(rx);
            (instrument, stats, start.elapsed())
        });
        matcher_handles.push(handle);
    }

    let senders = Arc::new(senders);
    // Timer starts on the first batch received from any client (not on connection accept)
    let first_batch_time: Arc<OnceLock<Instant>> = Arc::new(OnceLock::new());

    // Accept client connections
    let mut client_handles = Vec::with_capacity(expected_clients);

    for client_idx in 0..expected_clients {
        let (stream, addr) = listener.accept().expect("accept failed");
        println!("Client {} connected from {}", client_idx, addr);

        let senders = Arc::clone(&senders);
        let first_batch_time = Arc::clone(&first_batch_time);

        let handle = thread::spawn(move || handle_client(client_idx, stream, &senders, &first_batch_time));
        client_handles.push(handle);
    }

    println!();
    println!("All {} clients connected. Processing orders...", expected_clients);
    println!();

    // Wait for all client handlers to finish
    let mut total_routed = 0u64;
    let mut total_gateway_rejected = 0u64;

    for handle in client_handles {
        let (routed, rejected) = handle.join().expect("client handler panicked");
        total_routed += routed;
        total_gateway_rejected += rejected;
    }

    // All clients done — drop senders to signal matchers to drain and exit
    drop(senders);

    // Collect matcher results — timer started on first batch received
    let elapsed = first_batch_time
        .get()
        .map(|t| t.elapsed())
        .unwrap_or_default();
    let elapsed_secs = elapsed.as_secs_f64();

    let mut total_trades = 0u64;
    let mut total_volume = 0u64;
    let mut total_received = 0u64;
    let mut total_fully_filled = 0u64;
    let mut total_partially_filled = 0u64;
    let mut total_queued = 0u64;
    let mut total_matcher_rejected = 0u64;

    println!("=========================");
    println!("Per-Instrument Statistics");
    println!("=========================");

    for handle in matcher_handles {
        let (instrument, stats, match_time) = handle.join().expect("matcher thread panicked");
        let match_secs = match_time.as_secs_f64();
        let match_rate = stats.orders_received as f64 / match_secs;

        println!("\n{}", instrument);
        println!("  Orders received:    {}", stats.orders_received);
        println!("  Trades executed:    {}", stats.total_trades);
        println!("  Volume matched:     {} units", stats.volume_matched);
        if stats.orders_received > 0 {
            println!(
                "  Fully filled:       {} ({:.1}%)",
                stats.orders_fully_filled,
                (stats.orders_fully_filled as f64 / stats.orders_received as f64) * 100.0
            );
            println!(
                "  Partially filled:   {} ({:.1}%)",
                stats.orders_partially_filled,
                (stats.orders_partially_filled as f64 / stats.orders_received as f64) * 100.0
            );
            println!(
                "  Queued (no fill):   {} ({:.1}%)",
                stats.orders_queued,
                (stats.orders_queued as f64 / stats.orders_received as f64) * 100.0
            );
            println!(
                "  Rejected (matcher): {} ({:.1}%)",
                stats.orders_rejected,
                (stats.orders_rejected as f64 / stats.orders_received as f64) * 100.0
            );
        }
        println!("  ---");
        println!(
            "  Matcher time:       {:.3}s ({:.1}M orders/sec)",
            match_secs,
            match_rate / 1_000_000.0
        );

        total_trades += stats.total_trades;
        total_volume += stats.volume_matched;
        total_received += stats.orders_received;
        total_fully_filled += stats.orders_fully_filled;
        total_partially_filled += stats.orders_partially_filled;
        total_queued += stats.orders_queued;
        total_matcher_rejected += stats.orders_rejected;
    }

    let orders_per_sec = (total_routed + total_gateway_rejected) as f64 / elapsed_secs;

    println!();
    println!("=========================");
    println!("Gateway Statistics");
    println!("=========================");
    println!("Orders routed to matchers: {}", total_routed);
    println!("Orders rejected at gateway: {}", total_gateway_rejected);
    println!(
        "Total orders received:     {}",
        total_routed + total_gateway_rejected
    );

    println!();
    println!("=========================");
    println!("Aggregate Matching Statistics");
    println!("=========================");
    println!("Total orders matched:    {}", total_received);
    println!("Total trades executed:   {}", total_trades);
    println!("Total volume matched:    {} units", total_volume);
    if total_received > 0 {
        println!(
            "Orders fully filled:     {} ({:.1}%)",
            total_fully_filled,
            (total_fully_filled as f64 / total_received as f64) * 100.0
        );
        println!(
            "Orders partially filled: {} ({:.1}%)",
            total_partially_filled,
            (total_partially_filled as f64 / total_received as f64) * 100.0
        );
        println!(
            "Orders queued (no fill): {} ({:.1}%)",
            total_queued,
            (total_queued as f64 / total_received as f64) * 100.0
        );
        println!(
            "Orders rejected (matcher): {} ({:.1}%)",
            total_matcher_rejected,
            (total_matcher_rejected as f64 / total_received as f64) * 100.0
        );
    }

    println!();
    println!("=========================");
    println!("Performance");
    println!("=========================");
    println!("Wall clock time:   {:.3}s", elapsed_secs);
    println!("Throughput:        {:.1}M orders/sec", orders_per_sec / 1_000_000.0);
    println!("=========================");
}

/// Handle a single client connection: read batches, validate, route by instrument.
fn handle_client(
    client_idx: usize,
    stream: TcpStream,
    senders: &[crossbeam_channel::Sender<OrderBatch>],
    first_batch_time: &OnceLock<Instant>,
) -> (u64, u64) {
    stream.set_nodelay(true).ok();
    let mut reader = BufReader::new(stream);

    let mut routed = 0u64;
    let mut rejected = 0u64;

    loop {
        match protocol::read_batch(&mut reader) {
            Ok(None) => break, // Shutdown sentinel or EOF
            Ok(Some(batch)) => {
                // Start the wall-clock timer on the very first batch from any client
                first_batch_time.get_or_init(Instant::now);
                // Gateway validation: split valid orders by instrument
                let mut per_instrument: [Vec<order_book::order::Order>; NUM_INSTRUMENTS] =
                    std::array::from_fn(|_| Vec::new());

                for order in batch {
                    if order.validate().is_err() {
                        rejected += 1;
                        continue;
                    }
                    per_instrument[order.instrument.as_usize()].push(order);
                }

                // Route each instrument's orders to the correct matcher channel
                for (inst_idx, orders) in per_instrument.into_iter().enumerate() {
                    if !orders.is_empty() {
                        routed += orders.len() as u64;
                        if senders[inst_idx].send(orders).is_err() {
                            eprintln!(
                                "Client {}: Matcher for instrument {} closed early",
                                client_idx,
                                Instrument::new(inst_idx as u8)
                                    .map(|i| format!("{}", i))
                                    .unwrap_or_else(|| "UNKNOWN".to_string())
                            );
                            return (routed, rejected);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Client {}: Error reading: {}", client_idx, e);
                break;
            }
        }
    }

    println!(
        "Client {} disconnected. Routed: {}, Rejected at gateway: {}",
        client_idx, routed, rejected
    );
    (routed, rejected)
}
