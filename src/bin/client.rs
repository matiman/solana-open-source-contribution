use order_book::{
    generator::BATCH_SIZE,
    instrument::{Instrument, NUM_INSTRUMENTS},
    order::{Order, Side},
    protocol,
};
use rand::{Rng, SeedableRng};
use std::io::BufWriter;
use std::net::TcpStream;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_ADDR: &str = "127.0.0.1:9000";
const DEFAULT_COUNT: u32 = 1_000_000;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let addr = args.get(1).map(|s| s.as_str()).unwrap_or(DEFAULT_ADDR);
    let count: u32 = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_COUNT);

    println!("Order Book Client");
    println!("=================");
    println!("Connecting to {}", addr);
    println!("Orders to send: {} (mixed instruments)", count);

    let stream = TcpStream::connect(addr).expect("failed to connect to gateway");
    stream.set_nodelay(true).ok();
    let mut writer = BufWriter::new(stream);

    // SmallRng (Xoshiro256++) — fast non-crypto RNG, ~4x faster than StdRng
    let mut rng = rand::rngs::SmallRng::from_entropy();

    let mut batch = Vec::with_capacity(BATCH_SIZE);
    let mut batches_sent = 0u64;

    let start = Instant::now();

    for id in 1..=count {
        let side = if rng.gen_bool(0.5) {
            Side::Buy
        } else {
            Side::Sell
        };

        // Random instrument (0..NUM_INSTRUMENTS)
        let instrument = Instrument::new(rng.gen_range(0..NUM_INSTRUMENTS as u8)).unwrap();

        // Occasionally generate invalid orders (1% of the time)
        let (price, quantity): (u32, u32) = if rng.gen_bool(0.01) {
            match rng.gen_range(0..2) {
                0 => (rng.gen_range(10_000..=20_000), 0), // Zero quantity
                _ => (0, rng.gen_range(1..=10)),           // Zero price
            }
        } else {
            (rng.gen_range(10_000u32..=20_000), rng.gen_range(1u32..=10))
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        let order = Order {
            timestamp,
            id,
            price,
            quantity,
            instrument,
            side,
        };

        batch.push(order);

        if batch.len() >= BATCH_SIZE {
            protocol::write_batch(&mut writer, &batch).expect("failed to write batch");
            batch.clear();
            batches_sent += 1;
        }
    }

    // Send final partial batch
    if !batch.is_empty() {
        protocol::write_batch(&mut writer, &batch).expect("failed to write final batch");
        batches_sent += 1;
    }

    // Shutdown sentinel
    protocol::write_shutdown(&mut writer).expect("failed to write shutdown");

    // Flush to ensure all data is sent
    use std::io::Write;
    writer.flush().expect("failed to flush");

    let elapsed = start.elapsed();
    let rate = count as f64 / elapsed.as_secs_f64() / 1_000_000.0;

    println!();
    println!("Done!");
    println!("Orders sent:  {}", count);
    println!("Batches sent: {}", batches_sent);
    println!("Time:         {:.3}s", elapsed.as_secs_f64());
    println!("Rate:         {:.1}M orders/sec", rate);
}
