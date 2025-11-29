mod generator;
mod matcher;
mod order;
mod order_book;

use tokio::sync::mpsc;
use std::time::Instant;

#[tokio::main]
async fn main() {
    println!("🚀 Order Book Matching Engine - MVP");
    println!("─────────────────────────────────────\n");

    // Create bounded mpsc channel with buffer size 1000
    let (tx, rx) = mpsc::channel(1000);

    // Start timing
    let start = Instant::now();

    // Spawn generator task - generate 1000 orders
    let generator_handle = tokio::spawn(async move {
        generator::generate_orders(tx, 1000).await;
    });

    // Spawn matcher task - process orders from channel
    let matcher_handle = tokio::spawn(async move {
        matcher::run_matcher(rx).await;
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(generator_handle, matcher_handle);

    // Calculate elapsed time
    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let orders_per_sec = 1000.0 / elapsed_secs;

    // Print summary statistics
    println!("\n─────────────────────────────────────");
    println!("📊 Summary Statistics");
    println!("─────────────────────────────────────");
    println!("Total orders processed: 1000");
    println!("Elapsed time: {:.3}s", elapsed_secs);
    println!("Throughput: {:.0} orders/sec", orders_per_sec);
    println!("─────────────────────────────────────");
}
