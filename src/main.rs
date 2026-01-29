mod generator;
mod matcher;
mod order;
mod order_book;

use std::time::Instant;

const TOTAL_ORDERS: u64 = 100_000_000;
const CHANNEL_BUFFER_SIZE: usize = 10_000;

fn main() {
    println!("🚀 High Performance Order Book Matching Engine");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Processing {} orders...\n", TOTAL_ORDERS);

    // Create bounded channel with buffer size 10,000
    let (tx, rx) = crossbeam_channel::bounded(CHANNEL_BUFFER_SIZE);

    // Start timing
    let start = Instant::now();
    let gen_start = Instant::now();

    // Spawn generator thread
    let generator_handle = std::thread::spawn(move || {
        generator::generate_orders(tx, TOTAL_ORDERS);
    });

    let match_start = Instant::now();
    // Spawn matcher thread
    let matcher_handle = std::thread::spawn(move || matcher::run_matcher(rx));

    // Wait for both threads to complete
    generator_handle.join().expect("Generator thread panicked");
    let gen_time = gen_start.elapsed();

    let stats = matcher_handle.join().expect("Matcher thread panicked");
    let match_time = match_start.elapsed();

    // Calculate elapsed time
    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let orders_per_sec = TOTAL_ORDERS as f64 / elapsed_secs;
    let match_rate = (stats.matched as f64 / TOTAL_ORDERS as f64) * 100.0;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Summary Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total orders processed: {}", TOTAL_ORDERS);
    println!(
        "Matched orders:         {} ({:.1}%)",
        stats.matched, match_rate
    );
    println!("Queued orders:          {}", stats.queued);
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
        "Matcher time:           {:.3}s ({:.1}%)",
        match_time.as_secs_f64(),
        (match_time.as_secs_f64() / elapsed_secs) * 100.0
    );
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
