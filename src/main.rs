use order_book::{generator,matching_engine::MatchingEngine};
use std::time::Instant;

const TOTAL_ORDERS: u64 = 100_000_000;
const CHANNEL_BUFFER_SIZE: usize = 10_000;

fn main() {
    println!("🚀 High Performance Order Book Matching Engine");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Processing {} orders...\n", TOTAL_ORDERS);

    let (tx, rx) = crossbeam_channel::bounded(CHANNEL_BUFFER_SIZE);

    let start = Instant::now();
    let gen_start = Instant::now();

    let generator_handle = std::thread::spawn(move || {
        generator::generate_orders(tx, TOTAL_ORDERS, 1);
    });

    let match_start = Instant::now();
    let engine_handle = std::thread::spawn(move || MatchingEngine::run(rx));

    generator_handle.join().expect("Generator thread panicked");
    let gen_time = gen_start.elapsed();

    let stats = engine_handle.join().expect("Matching engine thread panicked");
    let match_time = match_start.elapsed();

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let orders_per_sec = TOTAL_ORDERS as f64 / elapsed_secs;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Summary Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total orders processed: {}", TOTAL_ORDERS);
    println!("Total trades executed:  {}", stats.total_trades);
    println!("Volume matched:         {} units", stats.volume_matched);
    println!();
    println!(
        "Orders fully filled:    {} ({:.1}%)",
        stats.orders_fully_filled,
        (stats.orders_fully_filled as f64 / stats.orders_received as f64) * 100.0
    );
    println!(
        "Orders partially filled: {} ({:.1}%)",
        stats.orders_partially_filled,
        (stats.orders_partially_filled as f64 / stats.orders_received as f64) * 100.0
    );
    println!(
        "Orders queued (no fill): {} ({:.1}%)",
        stats.orders_queued,
        (stats.orders_queued as f64 / stats.orders_received as f64) * 100.0
    );
    println!(
        "Orders rejected (invalid): {} ({:.1}%)",
        stats.orders_rejected,
        (stats.orders_rejected as f64 / stats.orders_received as f64) * 100.0
    );
    println!();
    println!("Elapsed time:           {:.3}s", elapsed_secs);
    println!("Throughput:             {:.0} orders/sec", orders_per_sec);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("⏱️  Performance Breakdown");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!(
        "Generator time:         {:.3}s ({:.1}%)",
        gen_time.as_secs_f64(),
        (gen_time.as_secs_f64() / elapsed_secs) * 100.0
    );
    println!(
        "Matching engine time:   {:.3}s ({:.1}%)",
        match_time.as_secs_f64(),
        (match_time.as_secs_f64() / elapsed_secs) * 100.0
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
