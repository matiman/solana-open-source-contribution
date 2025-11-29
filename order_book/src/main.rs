mod generator;
mod matcher;
mod order;
mod order_book;

use std::time::Instant;
use tokio::sync::mpsc;

const TOTAL_ORDERS: u64 = 1_000_000;
const CHANNEL_BUFFER_SIZE: usize = 10_000;

#[tokio::main]
async fn main() {
    println!("🚀 Order Book Matching Engine - High Performance");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Processing {} orders...\n", TOTAL_ORDERS);

    // Create bounded mpsc channel with buffer size 10,000
    let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

    // Start timing
    let start = Instant::now();

    // Spawn generator task - generate 1M orders
    let generator_handle = tokio::spawn(async move {
        generator::generate_orders(tx, TOTAL_ORDERS).await;
    });

    // Spawn matcher task - process orders from channel
    let matcher_handle = tokio::spawn(async move {
        matcher::run_matcher(rx).await
    });

    // Wait for both tasks to complete
    let (gen_result, matcher_result) = tokio::join!(generator_handle, matcher_handle);

    gen_result.expect("Generator task failed");
    let stats = matcher_result.expect("Matcher task failed");

    // Calculate elapsed time
    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let orders_per_sec = TOTAL_ORDERS as f64 / elapsed_secs;
    let match_rate = (stats.matched as f64 / TOTAL_ORDERS as f64) * 100.0;

    // Print summary statistics
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Summary Statistics");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Total orders processed: {}", TOTAL_ORDERS);
    println!("Matched orders:         {} ({:.1}%)", stats.matched, match_rate);
    println!("Queued orders:          {}", stats.queued);
    println!("Elapsed time:           {:.3}s", elapsed_secs);
    println!("Throughput:             {:.0} orders/sec", orders_per_sec);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
}
