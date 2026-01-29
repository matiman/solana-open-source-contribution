use crate::order::{Order, Side};
use crossbeam_channel::Sender;
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

const PRICE_LEVELS: [u64; 5] = [100, 101, 102, 103, 104];


pub fn generate_orders(tx: Sender<Order>, count: u64) {
    let mut rng = rand::rngs::StdRng::from_entropy();

    for id in 1..=count {
        // Random side (50/50)
        let side = if rng.gen_bool(0.5) {
            Side::Buy
        } else {
            Side::Sell
        };

        // Random price from the 5 price levels
        let price = PRICE_LEVELS[rng.gen_range(0..PRICE_LEVELS.len())];

        // Quantity always 1 for MVP
        let quantity = 1;

        // Timestamp from system time
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

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
            generate_orders(tx, 1);
        });

        let order = rx.recv().unwrap();
        assert!(
            order.price >= 100 && order.price <= 104,
            "Price {} should be in range [100, 104]",
            order.price
        );
    }

    #[test]
    fn test_generated_order_has_valid_side() {
        // Generate multiple orders and verify sides are valid
        let (tx, rx) = crossbeam_channel::unbounded();

        std::thread::spawn(move || {
            generate_orders(tx, 10);
        });

        for _ in 0..10 {
            let order = rx.recv().unwrap();
            // Side must be either Buy or Sell (this will compile only if valid)
            match order.side {
                Side::Buy | Side::Sell => {} // Valid
            }
        }
    }
}
