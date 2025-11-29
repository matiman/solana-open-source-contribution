use crate::order::Order;
use crate::order_book::OrderBook;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct MatcherStats {
    pub matched: u64,
    pub queued: u64,
}

#[allow(dead_code)]
pub async fn run_matcher(mut rx: mpsc::Receiver<Order>) -> MatcherStats {
    let mut order_book = OrderBook::new();
    let mut matched_count = 0u64;
    let mut queued_count = 0u64;

    while let Some(order) = rx.recv().await {
        match order_book.try_match(order.clone()) {
            Some((_buy, _sell)) => {
                matched_count += 1;
                // Comment out for performance - uncomment to see individual matches
                // println!(
                //     "MATCHED: Buy #{} <-> Sell #{} @ ${}",
                //     _buy.id, _sell.id, _buy.price
                // );
            }
            None => {
                queued_count += 1;
                // Comment out for performance - uncomment to see individual queued orders
                // println!(
                //     "QUEUED: {:?} #{} @ ${}",
                //     order.side, order.id, order.price
                // );
            }
        }
    }

    MatcherStats {
        matched: matched_count,
        queued: queued_count,
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
        let stats = run_matcher(rx).await;

        // If we get here, the matcher successfully processed the order
        // Verify stats are reasonable (1 order should be queued since no match)
        assert_eq!(stats.matched + stats.queued, 1);
    }
}
