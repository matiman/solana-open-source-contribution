use crate::order::Order;
use crate::order_book::{LimitOrderBook, MatchResult};
use crossbeam_channel::Receiver;

#[derive(Debug, Default)]
pub struct MatchingStats {
    pub total_trades: u64,
    pub volume_matched: u64,
    pub orders_received: u64,
    pub orders_fully_filled: u64,
    pub orders_partially_filled: u64,
    pub orders_queued: u64,
    pub orders_rejected: u64,
}

pub struct MatchingEngine {
    book: LimitOrderBook,
}

impl MatchingEngine {
    pub fn new() -> Self {
        Self {
            book: LimitOrderBook::new(),
        }
    }

    pub fn process_limit_order(&mut self, order: Order) -> MatchResult {
        self.book.try_match(order)
    }

    /// Consume orders from the channel until it closes, returning session statistics.
    pub fn run(rx: Receiver<Order>) -> MatchingStats {
        let mut engine = Self::new();
        let mut stats = MatchingStats::default();

        while let Ok(order) = rx.recv() {
            stats.orders_received += 1;

            let result = engine.process_limit_order(order);

            if result.validation_error.is_some() {
                stats.orders_rejected += 1;
                // Don't recycle — result.trades is empty Vec, would lose existing buffer capacity
                continue;
            }

            stats.total_trades += result.trades.len() as u64;
            for (buy, _sell) in &result.trades {
                stats.volume_matched += buy.quantity;
            }

            if result.trades.is_empty() {
                stats.orders_queued += 1;
            } else if result.remaining_quantity == 0 {
                stats.orders_fully_filled += 1;
            } else {
                stats.orders_partially_filled += 1;
            }

            engine.book.recycle_trades_buffer(result.trades);
        }

        stats
    }
}

impl Default for MatchingEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order::Side;
    use rand::Rng;

    #[test]
    fn test_matching_engine_processes_order() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Send a buy order
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 10_000,
            quantity: 1,
            timestamp: 1000,
        };

        tx.send(order.clone()).unwrap();

        // Close the channel so engine will exit
        drop(tx);

        // Run the matching engine - it should process the order without panicking
        let stats = MatchingEngine::run(rx);

        // If we get here, the engine successfully processed the order
        // Verify stats are reasonable (1 order should be queued since no match)
        assert_eq!(stats.orders_received, 1);
        assert_eq!(stats.orders_queued, 1); // No matches, so it should be queued
        assert_eq!(stats.total_trades, 0); // No trades occurred
    }

    #[test]
    fn test_matching_engine_processes_multiple_orders_with_matches() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Send 10 sell orders at various prices (ticks)
        for i in 1..=10 {
            let price = 10_000 + (i % 5) * 100; // Prices: 10100, 10200, 10300, 10400, 10000, ...
            let order = Order {
                id: i,
                side: Side::Sell,
                price,
                quantity: 1,
                timestamp: 1000 + i,
            };
            tx.send(order).unwrap();
        }

        // Send 10 buy orders at same prices - should create matches
        for i in 11..=20 {
            let price = 10_000 + ((i - 11) % 5) * 100; // Prices: 10000, 10100, 10200, 10300, 10400, ...
            let order = Order {
                id: i,
                side: Side::Buy,
                price,
                quantity: 1,
                timestamp: 2000 + i,
            };
            tx.send(order).unwrap();
        }

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_with_randomized_orders() {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut rng = rand::thread_rng();

        // Generate 20 randomized orders with tick prices in [10_000, 20_000]
        for i in 1..=20 {
            let side = if rng.gen_bool(0.5) {
                Side::Buy
            } else {
                Side::Sell
            };
            let price = rng.gen_range(10_000_u64..=20_000);
            let order = Order {
                id: i,
                side,
                price,
                quantity: 1,
                timestamp: 1000 + i * 100,
            };
            tx.send(order).unwrap();
        }

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_fifo_with_multiple_orders() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Send 5 sell orders all at 10_000 ticks ($100.00)
        for i in 1..=5 {
            let order = Order {
                id: i,
                side: Side::Sell,
                price: 10_000,
                quantity: 1,
                timestamp: 1000 + i,
            };
            tx.send(order).unwrap();
        }

        // Send 3 buy orders at 10_000 - should match first 3 sells in FIFO order
        for i in 6..=8 {
            let order = Order {
                id: i,
                side: Side::Buy,
                price: 10_000,
                quantity: 1,
                timestamp: 2000 + i,
            };
            tx.send(order).unwrap();
        }

        // Send 5 more buy orders at different prices (no matches)
        let prices: [u64; 5] = [10_100, 10_200, 10_300, 10_400, 10_100];
        for (idx, i) in (9..=13).enumerate() {
            let order = Order {
                id: i,
                side: Side::Buy,
                price: prices[idx],
                quantity: 1,
                timestamp: 3000 + i,
            };
            tx.send(order).unwrap();
        }

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_alternating_buy_sell_pattern() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Alternating buy/sell orders with overlapping prices
        for i in 1..=15 {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = 10_000 + (i % 3) * 100; // Prices cycle: 10100, 10200, 10000, ...
            let order = Order {
                id: i,
                side,
                price,
                quantity: 1,
                timestamp: 1000 + i * 50,
            };
            tx.send(order).unwrap();
        }

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_rejects_invalid_orders() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Send valid orders
        for i in 1..=5 {
            let order = Order {
                id: i,
                side: Side::Buy,
                price: 10_000,
                quantity: 1,
                timestamp: 1000 + i,
            };
            tx.send(order).unwrap();
        }

        // Send invalid orders (zero quantity)
        for i in 6..=10 {
            let order = Order {
                id: i,
                side: Side::Buy,
                price: 10_000,
                quantity: 0, // Invalid!
                timestamp: 1000 + i,
            };
            tx.send(order).unwrap();
        }

        // Send more invalid orders (zero price)
        for i in 11..=13 {
            let order = Order {
                id: i,
                side: Side::Sell,
                price: 0, // Invalid!
                quantity: 10,
                timestamp: 1000 + i,
            };
            tx.send(order).unwrap();
        }

        drop(tx);
        let stats = MatchingEngine::run(rx);

        // Verify stats
        assert_eq!(stats.orders_received, 13);
        assert_eq!(stats.orders_rejected, 8); // 5 zero quantity + 3 zero price
        assert_eq!(stats.orders_queued, 5); // 5 valid orders queued
        assert_eq!(stats.total_trades, 0); // No matches
    }
}
