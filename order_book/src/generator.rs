use crate::order::{Order, Side};
use rand::{Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc;

const PRICE_LEVELS: [u64; 5] = [100, 101, 102, 103, 104];


pub async fn generate_orders(tx: mpsc::Sender<Order>, count: u64) {
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
        if tx.send(order).await.is_err() {
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
        let (tx, mut rx) = mpsc::channel(10);

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            generate_orders(tx, 1).await;

            let order = rx.recv().await.unwrap();
            assert!(
                order.price >= 100 && order.price <= 104,
                "Price {} should be in range [100, 104]",
                order.price
            );
        });
    }

    #[test]
    fn test_generated_order_has_valid_side() {
        // Generate multiple orders and verify sides are valid
        let (tx, mut rx) = mpsc::channel(100);

        tokio::runtime::Runtime::new().unwrap().block_on(async {
            generate_orders(tx, 10).await;

            for _ in 0..10 {
                let order = rx.recv().await.unwrap();
                // Side must be either Buy or Sell (this will compile only if valid)
                match order.side {
                    Side::Buy | Side::Sell => {} // Valid
                }
            }
        });
    }
}
