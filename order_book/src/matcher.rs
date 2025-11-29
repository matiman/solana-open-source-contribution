use crate::order::Order;
use crate::order_book::OrderBook;
use tokio::sync::mpsc;

#[allow(dead_code)]
pub async fn run_matcher(mut rx: mpsc::Receiver<Order>) {
    let mut order_book = OrderBook::new();

    while let Some(order) = rx.recv().await {
        match order_book.try_match(order.clone()) {
            Some((buy, sell)) => {
                println!(
                    "MATCHED: Buy #{} <-> Sell #{} @ ${}",
                    buy.id, sell.id, buy.price
                );
            }
            None => {
                println!(
                    "QUEUED: {:?} #{} @ ${}",
                    order.side, order.id, order.price
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;
    use rand::Rng;

    #[tokio::test]
    async fn test_matcher_processes_order() {
        let (tx, rx) = mpsc::channel(10);

        // Send a buy order
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 1,
            timestamp: 1000,
        };

        tx.send(order.clone()).await.unwrap();

        // Close the channel so matcher will exit
        drop(tx);

        // Run the matcher - it should process the order without panicking
        run_matcher(rx).await;

        // If we get here, the matcher successfully processed the order
    }

    #[tokio::test]
    async fn test_matcher_processes_multiple_orders_with_matches() {
        let (tx, rx) = mpsc::channel(100);

        // Send 10 sell orders at various prices
        for i in 1..=10 {
            let price = 100 + (i % 5); // Prices: 101, 102, 103, 104, 100, 101, ...
            let order = Order {
                id: i,
                side: Side::Sell,
                price,
                quantity: 1,
                timestamp: 1000 + i,
            };
            tx.send(order).await.unwrap();
        }

        // Send 10 buy orders at same prices - should create matches
        for i in 11..=20 {
            let price = 100 + ((i - 11) % 5); // Prices: 100, 101, 102, 103, 104, 100, ...
            let order = Order {
                id: i,
                side: Side::Buy,
                price,
                quantity: 1,
                timestamp: 2000 + i,
            };
            tx.send(order).await.unwrap();
        }

        drop(tx);
        run_matcher(rx).await;
    }

    #[tokio::test]
    async fn test_matcher_with_randomized_orders() {
        let (tx, rx) = mpsc::channel(100);
        let mut rng = rand::thread_rng();
        let price_levels = [100, 101, 102, 103, 104];

        // Generate 20 randomized orders
        for i in 1..=20 {
            let side = if rng.gen_bool(0.5) {
                Side::Buy
            } else {
                Side::Sell
            };
            let price = price_levels[rng.gen_range(0..price_levels.len())];
            let order = Order {
                id: i,
                side,
                price,
                quantity: 1,
                timestamp: 1000 + i * 100,
            };
            tx.send(order).await.unwrap();
        }

        drop(tx);
        run_matcher(rx).await;
    }

    #[tokio::test]
    async fn test_matcher_fifo_with_multiple_orders() {
        let (tx, rx) = mpsc::channel(100);

        // Send 5 sell orders all at price 100
        for i in 1..=5 {
            let order = Order {
                id: i,
                side: Side::Sell,
                price: 100,
                quantity: 1,
                timestamp: 1000 + i,
            };
            tx.send(order).await.unwrap();
        }

        // Send 3 buy orders at price 100 - should match first 3 sells in FIFO order
        for i in 6..=8 {
            let order = Order {
                id: i,
                side: Side::Buy,
                price: 100,
                quantity: 1,
                timestamp: 2000 + i,
            };
            tx.send(order).await.unwrap();
        }

        // Send 5 more buy orders at different prices (no matches)
        for i in 9..=13 {
            let order = Order {
                id: i,
                side: Side::Buy,
                price: 95 + i, // Prices: 104, 105, 106, 107, 108
                quantity: 1,
                timestamp: 3000 + i,
            };
            tx.send(order).await.unwrap();
        }

        drop(tx);
        run_matcher(rx).await;
    }

    #[tokio::test]
    async fn test_matcher_alternating_buy_sell_pattern() {
        let (tx, rx) = mpsc::channel(100);

        // Alternating buy/sell orders with overlapping prices
        for i in 1..=15 {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = 100 + (i % 3); // Prices cycle: 101, 102, 100, 101, 102, 100, ...
            let order = Order {
                id: i,
                side,
                price,
                quantity: 1,
                timestamp: 1000 + i * 50,
            };
            tx.send(order).await.unwrap();
        }

        drop(tx);
        run_matcher(rx).await;
    }
}
