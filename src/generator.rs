use crate::order::{Order, Side};
use crossbeam_channel::Sender;
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

// Price range in ticks (1 tick = $0.01)
// $100.00 = 10_000 ticks, $200.00 = 20_000 ticks
const MIN_PRICE: u64 = 10_000;
const MAX_PRICE: u64 = 20_000;


pub fn generate_orders(tx: Sender<Order>, count: u64, start_id: u64) {
    // SmallRng (Xoshiro256++) — fast non-crypto RNG, ~4x faster than StdRng (ChaCha20)
    let mut rng = rand::rngs::SmallRng::from_entropy();

    for id in start_id..start_id + count {
        // Random side (50/50)
        let side = if rng.gen_bool(0.5) {
            Side::Buy
        } else {
            Side::Sell
        };

        // Occasionally generate invalid orders to test validation (1% of the time)
        let (price, quantity) = if rng.gen_bool(0.01) {
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
            id,
            side,
            price,
            quantity,
            timestamp,
        };

        // Send order through channel
        if tx.send(order).is_err() {
            // Receiver dropped, stop generating
            break;
        }
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
            generate_orders(tx, 1, 1);
        });

        let order = rx.recv().unwrap();
        assert!(
            order.price >= MIN_PRICE && order.price <= MAX_PRICE,
            "Price {} should be in range [{}, {}]",
            order.price, MIN_PRICE, MAX_PRICE
        );
    }

    #[test]
    fn test_generated_order_has_valid_side() {
        // Generate multiple orders and verify sides are valid
        let (tx, rx) = crossbeam_channel::unbounded();

        std::thread::spawn(move || {
            generate_orders(tx, 10, 1);
        });

        for _ in 0..10 {
            let order = rx.recv().unwrap();
            // Side must be either Buy or Sell (this will compile only if valid)
            match order.side {
                Side::Buy | Side::Sell => {} // Valid
            }
        }
    }

    #[test]
    fn test_smart_constructor_usage() {
        // Demonstrate the smart constructor pattern
        // This is how orders SHOULD be created in production code
        let valid_result = Order::new(
            1,
            Side::Buy,
            15_050, // $150.50
            5,
            1000,
        );
        assert!(valid_result.is_ok());

        let invalid_result = Order::new(
            2,
            Side::Sell,
            0,  // Invalid price!
            10,
            2000,
        );
        assert!(invalid_result.is_err());
    }
}
