use crate::instrument::Instrument;
use crate::order::{Order, Side};
use crossbeam_channel::Sender;
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

// Price range in ticks (1 tick = $0.01)
// $100.00 = 10_000 ticks, $200.00 = 20_000 ticks
const MIN_PRICE: u32 = 10_000;
const MAX_PRICE: u32 = 20_000;

/// Batch size for order generation. 1024 orders × 24 bytes = 24KB per batch (fits in L1 cache).
pub const BATCH_SIZE: usize = 1024;

/// Order batch type for channel communication
pub type OrderBatch = Vec<Order>;

pub fn generate_orders(tx: Sender<OrderBatch>, count: u32, start_id: u32, instrument: Instrument) {
    // SmallRng (Xoshiro256++) — fast non-crypto RNG, ~4x faster than StdRng (ChaCha20)
    let mut rng = rand::rngs::SmallRng::from_entropy();

    // Pre-allocate batch buffer
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for id in start_id..start_id.saturating_add(count) {
        // Random side (50/50)
        let side = if rng.gen_bool(0.5) {
            Side::Buy
        } else {
            Side::Sell
        };

        // Occasionally generate invalid orders to test validation (1% of the time)
        let (price, quantity): (u32, u32) = if rng.gen_bool(0.01) {
            // Generate invalid order
            let invalid_type = rng.gen_range(0..2);
            match invalid_type {
                0 => {
                    // Zero quantity
                    let price = rng.gen_range(MIN_PRICE..=MAX_PRICE);
                    (price, 0)
                }
                _ => {
                    // Zero price
                    let quantity = rng.gen_range(1..=10);
                    (0, quantity)
                }
            }
        } else {
            // Generate valid order
            let price = rng.gen_range(MIN_PRICE..=MAX_PRICE);
            let quantity = rng.gen_range(1..=10);
            (price, quantity)
        };

        // Nanoseconds since Unix epoch — the standard in exchanges (CME, NYSE, NASDAQ).
        // The generator acts as the gateway: timestamps are stamped at the boundary,
        // so the matching engine never needs to call a clock.
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        // Create order without validation (for testing the matcher's validation)
        // NOTE: In production, you would use Order::new() smart constructor
        // which validates at creation time. We intentionally bypass it here
        // to test that the matcher correctly rejects invalid orders.
        let order = Order {
            timestamp,
            id,
            price,
            quantity,
            instrument,
            side,
        };

        batch.push(order);

        // Send batch when full
        if batch.len() >= BATCH_SIZE {
            if tx.send(batch).is_err() {
                // Receiver dropped, stop generating
                return;
            }
            batch = Vec::with_capacity(BATCH_SIZE);
        }
    }

    // Send final partial batch if any orders remain
    if !batch.is_empty() {
        let _ = tx.send(batch);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generated_order_has_valid_price() {
        // Generate a single order and verify price is in valid range
        let (tx, rx) = crossbeam_channel::unbounded();

        std::thread::spawn(move || {
            generate_orders(tx, 1, 1, Instrument::BTCUSDC);
        });

        // Receive batch and get first order
        let batch = rx.recv().unwrap();
        let order = &batch[0];
        // Price can be 0 for invalid test orders (1% chance), so we check OR
        assert!(
            order.price == 0 || (order.price >= MIN_PRICE && order.price <= MAX_PRICE),
            "Price {} should be 0 (invalid) or in range [{}, {}]",
            order.price, MIN_PRICE, MAX_PRICE
        );
    }

    #[test]
    fn test_generated_order_has_valid_side() {
        // Generate multiple orders and verify sides are valid
        let (tx, rx) = crossbeam_channel::unbounded();

        std::thread::spawn(move || {
            generate_orders(tx, 10, 1, Instrument::BTCUSDC);
        });

        // Receive batch and check all orders
        let batch = rx.recv().unwrap();
        assert_eq!(batch.len(), 10);
        for order in batch {
            // Side must be either Buy or Sell (this will compile only if valid)
            match order.side {
                Side::Buy | Side::Sell => {} // Valid
            }
        }
    }

    #[test]
    fn test_batch_size_honored() {
        // Generate more than BATCH_SIZE orders and verify batching
        let (tx, rx) = crossbeam_channel::unbounded();
        let num_orders = BATCH_SIZE as u32 + 100;

        std::thread::spawn(move || {
            generate_orders(tx, num_orders, 1, Instrument::BTCUSDC);
        });

        // First batch should be exactly BATCH_SIZE
        let batch1 = rx.recv().unwrap();
        assert_eq!(batch1.len(), BATCH_SIZE);

        // Second batch should be the remainder (100)
        let batch2 = rx.recv().unwrap();
        assert_eq!(batch2.len(), 100);

        // No more batches
        assert!(rx.recv().is_err());
    }

    #[test]
    fn test_smart_constructor_usage() {
        // Demonstrate the smart constructor pattern
        // This is how orders SHOULD be created in production code
        let valid_result = Order::new(
            1,                       // id: u32
            Instrument::BTCUSDC,     // instrument
            Side::Buy,
            15_050,                  // $150.50 - price: u32
            5,                       // quantity: u32
            1000,                    // timestamp: u64
        );
        assert!(valid_result.is_ok());

        let invalid_result = Order::new(
            2,                       // id: u32
            Instrument::BTCUSDC,     // instrument
            Side::Sell,
            0,                       // Invalid price!
            10,                      // quantity: u32
            2000,                    // timestamp: u64
        );
        assert!(invalid_result.is_err());
    }
}
