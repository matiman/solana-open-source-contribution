use crate::generator::OrderBatch;
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

    /// Process a limit order. Trades are written to the caller-provided buffer.
    pub fn process_limit_order(
        &mut self,
        order: Order,
        trades: &mut Vec<(Order, Order)>,
    ) -> MatchResult {
        self.book.try_match(order, trades)
    }

    /// Consume order batches from the channel until it closes, returning session statistics.
    pub fn run(rx: Receiver<OrderBatch>) -> MatchingStats {
        let mut engine = Self::new();
        let mut stats = MatchingStats::default();
        let mut trades = Vec::new(); // Caller-owned buffer, reused across all orders

        // Outer loop: receive batches
        while let Ok(batch) = rx.recv() {
            // Inner loop: process each order in the batch
            for order in batch {
                stats.orders_received += 1;
                trades.clear(); // Caller controls buffer lifecycle

                let result = engine.process_limit_order(order, &mut trades);

                if result.validation_error.is_some() {
                    stats.orders_rejected += 1;
                    continue;
                }

                stats.total_trades += result.trade_count as u64;
                for (buy, _sell) in &trades {
                    stats.volume_matched += buy.quantity as u64;
                }

                if result.trade_count == 0 {
                    stats.orders_queued += 1;
                } else if result.remaining_quantity == 0 {
                    stats.orders_fully_filled += 1;
                } else {
                    stats.orders_partially_filled += 1;
                }
            }
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

        // Send a buy order as a single-order batch
        let order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 1,
            instrument: 0,
            side: Side::Buy,
        };

        tx.send(vec![order]).unwrap();

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

        // Collect sell orders into a batch
        let mut sell_batch = Vec::new();
        for i in 1u32..=10 {
            let price = 10_000 + (i % 5) * 100; // Prices: 10100, 10200, 10300, 10400, 10000, ...
            let order = Order {
                timestamp: 1000 + i as u64,
                id: i,
                price,
                quantity: 1,
                instrument: 0,
                side: Side::Sell,
            };
            sell_batch.push(order);
        }
        tx.send(sell_batch).unwrap();

        // Collect buy orders into a batch
        let mut buy_batch = Vec::new();
        for i in 11u32..=20 {
            let price = 10_000 + ((i - 11) % 5) * 100; // Prices: 10000, 10100, 10200, 10300, 10400, ...
            let order = Order {
                timestamp: 2000 + i as u64,
                id: i,
                price,
                quantity: 1,
                instrument: 0,
                side: Side::Buy,
            };
            buy_batch.push(order);
        }
        tx.send(buy_batch).unwrap();

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_with_randomized_orders() {
        let (tx, rx) = crossbeam_channel::unbounded();
        let mut rng = rand::thread_rng();

        // Generate 20 randomized orders into a batch
        let mut batch = Vec::new();
        for i in 1u32..=20 {
            let side = if rng.gen_bool(0.5) {
                Side::Buy
            } else {
                Side::Sell
            };
            let price = rng.gen_range(10_000_u32..=20_000);
            let order = Order {
                timestamp: 1000 + i as u64 * 100,
                id: i,
                price,
                quantity: 1,
                instrument: 0,
                side,
            };
            batch.push(order);
        }
        tx.send(batch).unwrap();

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_fifo_with_multiple_orders() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Collect all orders into a single batch for proper FIFO testing
        let mut batch = Vec::new();

        // 5 sell orders all at 10_000 ticks ($100.00)
        for i in 1u32..=5 {
            let order = Order {
                timestamp: 1000 + i as u64,
                id: i,
                price: 10_000,
                quantity: 1,
                instrument: 0,
                side: Side::Sell,
            };
            batch.push(order);
        }

        // 3 buy orders at 10_000 - should match first 3 sells in FIFO order
        for i in 6u32..=8 {
            let order = Order {
                timestamp: 2000 + i as u64,
                id: i,
                price: 10_000,
                quantity: 1,
                instrument: 0,
                side: Side::Buy,
            };
            batch.push(order);
        }

        // 5 more buy orders at different prices (no matches)
        let prices: [u32; 5] = [10_100, 10_200, 10_300, 10_400, 10_100];
        for (idx, i) in (9u32..=13).enumerate() {
            let order = Order {
                timestamp: 3000 + i as u64,
                id: i,
                price: prices[idx],
                quantity: 1,
                instrument: 0,
                side: Side::Buy,
            };
            batch.push(order);
        }

        tx.send(batch).unwrap();
        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_alternating_buy_sell_pattern() {
        let (tx, rx) = crossbeam_channel::unbounded();

        // Alternating buy/sell orders with overlapping prices
        let mut batch = Vec::new();
        for i in 1u32..=15 {
            let side = if i % 2 == 0 { Side::Buy } else { Side::Sell };
            let price = 10_000 + (i % 3) * 100; // Prices cycle: 10100, 10200, 10000, ...
            let order = Order {
                timestamp: 1000 + i as u64 * 50,
                id: i,
                price,
                quantity: 1,
                instrument: 0,
                side,
            };
            batch.push(order);
        }
        tx.send(batch).unwrap();

        drop(tx);
        MatchingEngine::run(rx);
    }

    #[test]
    fn test_matching_engine_rejects_invalid_orders() {
        let (tx, rx) = crossbeam_channel::unbounded();

        let mut batch = Vec::new();

        // Send valid orders
        for i in 1u32..=5 {
            let order = Order {
                timestamp: 1000 + i as u64,
                id: i,
                price: 10_000,
                quantity: 1,
                instrument: 0,
                side: Side::Buy,
            };
            batch.push(order);
        }

        // Send invalid orders (zero quantity)
        for i in 6u32..=10 {
            let order = Order {
                timestamp: 1000 + i as u64,
                id: i,
                price: 10_000,
                quantity: 0, // Invalid!
                instrument: 0,
                side: Side::Buy,
            };
            batch.push(order);
        }

        // Send more invalid orders (zero price)
        for i in 11u32..=13 {
            let order = Order {
                timestamp: 1000 + i as u64,
                id: i,
                price: 0, // Invalid!
                quantity: 10,
                instrument: 0,
                side: Side::Sell,
            };
            batch.push(order);
        }

        tx.send(batch).unwrap();
        drop(tx);
        let stats = MatchingEngine::run(rx);

        // Verify stats
        assert_eq!(stats.orders_received, 13);
        assert_eq!(stats.orders_rejected, 8); // 5 zero quantity + 3 zero price
        assert_eq!(stats.orders_queued, 5); // 5 valid orders queued
        assert_eq!(stats.total_trades, 0); // No matches
    }
}
