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
}
