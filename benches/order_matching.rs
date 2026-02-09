use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use order_book::order::{Order, Side};
use order_book::order_book::LimitOrderBook;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn generate_test_order(rng: &mut StdRng, id: u32) -> Order {
    let side = if rng.gen_bool(0.5) {
        Side::Buy
    } else {
        Side::Sell
    };
    // Price in ticks (1 tick = $0.01), range $100.00–$200.00
    let price = rng.gen_range(10_000_u32..=20_000);

    Order {
        timestamp: 1000 + id as u64,
        id,
        price,
        quantity: 1,
        instrument: 0,
        side,
    }
}

fn bench_try_match(c: &mut Criterion) {
    let mut group = c.benchmark_group("try_match");

    for size in [100u32, 1_000, 10_000, 100_000, 10_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut rng = StdRng::from_entropy();
            let mut order_book = LimitOrderBook::new();
            let mut trades = Vec::new();

            // Pre-populate order book with some orders
            for i in 0..size / 2 {
                let order = generate_test_order(&mut rng, i);
                order_book.add_order(order);
            }

            // Benchmark matching
            b.iter(|| {
                let order = generate_test_order(&mut rng, size);
                trades.clear();
                order_book.try_match(order, &mut trades);
            });
        });
    }
    group.finish();
}

fn bench_add_order(c: &mut Criterion) {
    let mut group = c.benchmark_group("add_order");

    for size in [100u32, 1_000, 10_000, 100_000, 10_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut rng = StdRng::from_entropy();
            let mut order_book = LimitOrderBook::new();

            b.iter(|| {
                let order = generate_test_order(&mut rng, size);
                order_book.add_order(order);
            });
        });
    }
    group.finish();
}

fn bench_order_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("order_generation");

    for size in [1_000u32, 10_000, 100_000, 1_000_000, 10_000_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut rng = StdRng::from_entropy();

            b.iter(|| {
                for i in 0..size {
                    let _order = generate_test_order(&mut rng, i);
                }
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_try_match,
    bench_add_order,
    bench_order_generation
);
criterion_main!(benches);
