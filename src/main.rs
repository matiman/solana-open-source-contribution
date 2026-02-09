use order_book::{
    generator,
    instrument::{self, ALL_INSTRUMENTS, NUM_INSTRUMENTS},
    matching_engine::{MatchingEngine, MatchingStats},
};
use std::thread;
use std::time::{Duration, Instant};

const ORDERS_PER_INSTRUMENT: u32 = 100_000_000;
const CHANNEL_BUFFER_SIZE: usize = 100; // Batches, not orders (100 × 1024 = 102K orders in flight)

struct InstrumentResult {
    instrument_id: u8,
    stats: MatchingStats,
    gen_time: Duration,
    match_time: Duration,
}

fn main() {
    let total_orders = ORDERS_PER_INSTRUMENT as u64 * NUM_INSTRUMENTS as u64;

    println!("High Performance Order Book Matching Engine");
    println!("============================================");
    let instrument_names: Vec<_> = ALL_INSTRUMENTS
        .iter()
        .map(|&id| instrument::name(id))
        .collect();
    println!("Instruments: {} ({})", NUM_INSTRUMENTS, instrument_names.join(", "));
    println!("Orders per instrument: {}", ORDERS_PER_INSTRUMENT);
    println!("Total orders: {}", total_orders);
    println!(
        "Threads: {} ({} generators + {} matchers)",
        NUM_INSTRUMENTS * 2,
        NUM_INSTRUMENTS,
        NUM_INSTRUMENTS
    );
    println!();

    let start = Instant::now();

    // Create channels and spawn threads for each instrument
    let mut generator_handles = Vec::with_capacity(NUM_INSTRUMENTS);
    let mut engine_handles = Vec::with_capacity(NUM_INSTRUMENTS);

    for &instrument_id in &ALL_INSTRUMENTS {
        let (tx, rx) = crossbeam_channel::bounded(CHANNEL_BUFFER_SIZE);

        // Each instrument gets its own ID range to avoid collisions
        let start_id = instrument_id as u32 * ORDERS_PER_INSTRUMENT + 1;

        // Spawn generator thread - returns elapsed time
        let gen_handle = thread::spawn(move || {
            let gen_start = Instant::now();
            generator::generate_orders(tx, ORDERS_PER_INSTRUMENT, start_id, instrument_id);
            gen_start.elapsed()
        });
        generator_handles.push((instrument_id, gen_handle));

        // Spawn matching engine thread - returns (stats, elapsed time)
        let engine_handle = thread::spawn(move || {
            let match_start = Instant::now();
            let stats = MatchingEngine::run(rx);
            (stats, match_start.elapsed())
        });
        engine_handles.push((instrument_id, engine_handle));
    }

    // Collect results
    let mut results: Vec<InstrumentResult> = Vec::with_capacity(NUM_INSTRUMENTS);

    // Wait for generators and collect their times
    let mut gen_times: Vec<(u8, Duration)> = Vec::with_capacity(NUM_INSTRUMENTS);
    for (instrument_id, handle) in generator_handles {
        let gen_time = handle.join().unwrap_or_else(|_| {
            panic!(
                "{} generator thread panicked",
                instrument::name(instrument_id)
            )
        });
        gen_times.push((instrument_id, gen_time));
    }

    // Wait for matchers and collect their stats + times
    for (instrument_id, handle) in engine_handles {
        let (stats, match_time) = handle.join().unwrap_or_else(|_| {
            panic!(
                "{} matching engine thread panicked",
                instrument::name(instrument_id)
            )
        });

        // Find corresponding generator time
        let gen_time = gen_times
            .iter()
            .find(|(id, _)| *id == instrument_id)
            .map(|(_, t)| *t)
            .unwrap();

        results.push(InstrumentResult {
            instrument_id,
            stats,
            gen_time,
            match_time,
        });
    }

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();

    // Aggregate stats
    let mut total_trades = 0u64;
    let mut total_volume = 0u64;
    let mut total_received = 0u64;
    let mut total_fully_filled = 0u64;
    let mut total_partially_filled = 0u64;
    let mut total_queued = 0u64;
    let mut total_rejected = 0u64;

    println!("============================================");
    println!("Per-Instrument Statistics");
    println!("============================================");

    for result in &results {
        let name = instrument::name(result.instrument_id);
        let stats = &result.stats;
        let gen_secs = result.gen_time.as_secs_f64();
        let match_secs = result.match_time.as_secs_f64();
        let gen_rate = ORDERS_PER_INSTRUMENT as f64 / gen_secs;
        let match_rate = stats.orders_received as f64 / match_secs;

        println!("\n{}", name);
        println!("  Orders received:    {}", stats.orders_received);
        println!("  Trades executed:    {}", stats.total_trades);
        println!("  Volume matched:     {} units", stats.volume_matched);
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
            "  Rejected (invalid): {} ({:.1}%)",
            stats.orders_rejected,
            (stats.orders_rejected as f64 / stats.orders_received as f64) * 100.0
        );
        println!("  ---");
        println!(
            "  Generator time:     {:.3}s ({:.1}M orders/sec)",
            gen_secs,
            gen_rate / 1_000_000.0
        );
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
        total_rejected += stats.orders_rejected;
    }

    let orders_per_sec = total_received as f64 / elapsed_secs;

    // Find slowest generator and matcher
    let slowest_gen = results
        .iter()
        .max_by(|a, b| a.gen_time.cmp(&b.gen_time))
        .unwrap();
    let slowest_match = results
        .iter()
        .max_by(|a, b| a.match_time.cmp(&b.match_time))
        .unwrap();

    println!();
    println!("============================================");
    println!("Aggregate Statistics");
    println!("============================================");
    println!("Total orders processed: {}", total_received);
    println!("Total trades executed:  {}", total_trades);
    println!("Total volume matched:   {} units", total_volume);
    println!();
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
        "Orders rejected:         {} ({:.1}%)",
        total_rejected,
        (total_rejected as f64 / total_received as f64) * 100.0
    );

    println!();
    println!("============================================");
    println!("Performance Breakdown");
    println!("============================================");
    println!(
        "Slowest generator: {} ({:.3}s)",
        instrument::name(slowest_gen.instrument_id),
        slowest_gen.gen_time.as_secs_f64()
    );
    println!(
        "Slowest matcher:   {} ({:.3}s)",
        instrument::name(slowest_match.instrument_id),
        slowest_match.match_time.as_secs_f64()
    );
    println!();
    println!("Wall clock time:   {:.3}s", elapsed_secs);
    println!("Throughput:        {:.1}M orders/sec", orders_per_sec / 1_000_000.0);
    println!("============================================");
}
