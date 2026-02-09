use crate::order::{Order, OrderValidationError, Side};
use std::collections::VecDeque;

// Instrument price range in ticks (circuit breaker bounds)
// $100.00 = 10,000 ticks, $200.00 = 20,000 ticks
pub const MIN_TICK: u32 = 10_000;
pub const MAX_TICK: u32 = 20_000;
const NUM_LEVELS: usize = (MAX_TICK - MIN_TICK + 1) as usize; // 10,001

/// Result of matching an order. Trades are written to the caller-provided buffer.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub trade_count: usize,            // Number of trades written to the buffer
    pub remaining_quantity: u32,       // Quantity that wasn't matched (added to book)
    pub validation_error: Option<OrderValidationError>,  // If order was invalid
}

pub struct LimitOrderBook {
    // Array-indexed price levels — true O(1) lookup with zero hashing overhead.
    // Index = price - MIN_TICK. Pre-allocated for the instrument's full tick range.
    // This is how production exchange matching engines work.
    buys: Vec<VecDeque<Order>>,
    sells: Vec<VecDeque<Order>>,
}

/// Convert a tick price to an array index
#[inline(always)]
fn price_index(price: u32) -> usize {
    (price - MIN_TICK) as usize
}

impl LimitOrderBook {
    pub fn new() -> Self {
        Self {
            buys: vec![VecDeque::new(); NUM_LEVELS],
            sells: vec![VecDeque::new(); NUM_LEVELS],
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let book = match order.side {
            Side::Buy => &mut self.buys,
            Side::Sell => &mut self.sells,
        };

        // Direct array index — one subtraction, one memory access, no hashing
        debug_assert!(order.price >= MIN_TICK && order.price <= MAX_TICK);
        book[price_index(order.price)].push_back(order);
    }

    #[cfg(test)]
    pub fn get_orders_at_price(&self, side: &Side, price: u32) -> Option<&VecDeque<Order>> {
        if price < MIN_TICK || price > MAX_TICK {
            return None;
        }
        let book = match side {
            Side::Buy => &self.buys,
            Side::Sell => &self.sells,
        };

        let queue = &book[price_index(price)];
        if queue.is_empty() {
            None
        } else {
            Some(queue)
        }
    }

    /// Match an incoming order against the book. Trades are appended to the caller-provided buffer.
    /// The caller owns the buffer and decides when to clear it (like `Read::read_to_end`).
    pub fn try_match(&mut self, mut order: Order, trades: &mut Vec<(Order, Order)>) -> MatchResult {
        let start_len = trades.len();

        // Validate order before processing
        if let Err(err) = order.validate() {
            return MatchResult {
                trade_count: 0,
                remaining_quantity: order.quantity,
                validation_error: Some(err),
            };
        }

        // Circuit breaker — reject orders outside the instrument's allowed price range
        if order.price < MIN_TICK || order.price > MAX_TICK {
            return MatchResult {
                trade_count: 0,
                remaining_quantity: order.quantity,
                validation_error: Some(OrderValidationError::PriceOutOfRange),
            };
        }

        let idx = price_index(order.price);

        match order.side {
            Side::Buy => {
                // Buy order: match against sells at same price
                let sell_queue = &mut self.sells[idx];
                while order.quantity > 0 {
                    if let Some(mut matched_sell) = sell_queue.pop_front() {
                        if matched_sell.quantity <= order.quantity {
                            order.quantity -= matched_sell.quantity;

                            let buy_trade = Order {
                                timestamp: order.timestamp,
                                id: order.id,
                                price: order.price,
                                quantity: matched_sell.quantity,
                                instrument: order.instrument,
                                side: order.side,
                            };
                            trades.push((buy_trade, matched_sell));
                        } else {
                            matched_sell.quantity -= order.quantity;

                            let sell_trade = Order {
                                timestamp: matched_sell.timestamp,
                                id: matched_sell.id,
                                price: matched_sell.price,
                                quantity: order.quantity,
                                instrument: matched_sell.instrument,
                                side: matched_sell.side,
                            };
                            let buy_trade = Order {
                                timestamp: order.timestamp,
                                id: order.id,
                                price: order.price,
                                quantity: order.quantity,
                                instrument: order.instrument,
                                side: order.side,
                            };
                            trades.push((buy_trade, sell_trade));

                            order.quantity = 0;
                            sell_queue.push_front(matched_sell);
                        }
                    } else {
                        break;
                    }
                }
            }
            Side::Sell => {
                // Sell order: match against buys at same price
                let buy_queue = &mut self.buys[idx];
                while order.quantity > 0 {
                    if let Some(mut matched_buy) = buy_queue.pop_front() {
                        if matched_buy.quantity <= order.quantity {
                            order.quantity -= matched_buy.quantity;

                            let sell_trade = Order {
                                timestamp: order.timestamp,
                                id: order.id,
                                price: order.price,
                                quantity: matched_buy.quantity,
                                instrument: order.instrument,
                                side: order.side,
                            };
                            trades.push((matched_buy, sell_trade));
                        } else {
                            matched_buy.quantity -= order.quantity;

                            let buy_trade = Order {
                                timestamp: matched_buy.timestamp,
                                id: matched_buy.id,
                                price: matched_buy.price,
                                quantity: order.quantity,
                                instrument: matched_buy.instrument,
                                side: matched_buy.side,
                            };
                            let sell_trade = Order {
                                timestamp: order.timestamp,
                                id: order.id,
                                price: order.price,
                                quantity: order.quantity,
                                instrument: order.instrument,
                                side: order.side,
                            };
                            trades.push((buy_trade, sell_trade));

                            order.quantity = 0;
                            buy_queue.push_front(matched_buy);
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        // Save remaining before moving order into the book (avoids clone)
        let remaining = order.quantity;
        if remaining > 0 {
            self.add_order(order);
        }

        MatchResult {
            trade_count: trades.len() - start_len,
            remaining_quantity: remaining,
            validation_error: None,
        }
    }
}

impl Default for LimitOrderBook {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_buy_order() {
        let mut book = LimitOrderBook::new();
        let order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000, // $100.00
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        book.add_order(order.clone());

        let orders = book.get_orders_at_price(&Side::Buy, 10_000);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0], order);
    }

    #[test]
    fn test_add_sell_order() {
        let mut book = LimitOrderBook::new();
        let order = Order {
            timestamp: 2000,
            id: 2,
            price: 10_400, // $104.00
            quantity: 5,
            instrument: 0,
            side: Side::Sell,
        };

        book.add_order(order.clone());

        let orders = book.get_orders_at_price(&Side::Sell, 10_400);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0], order);
    }

    #[test]
    fn test_multiple_orders_same_price() {
        let mut book = LimitOrderBook::new();

        let order1 = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let order2 = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 5,
            instrument: 0,
            side: Side::Buy,
        };

        book.add_order(order1.clone());
        book.add_order(order2.clone());

        let orders = book.get_orders_at_price(&Side::Buy, 10_000);
        assert!(orders.is_some());
        let orders_vec = orders.unwrap();
        assert_eq!(orders_vec.len(), 2);
        assert_eq!(orders_vec[0], order1);
        assert_eq!(orders_vec[1], order2);
    }

    #[test]
    fn test_orders_different_prices() {
        let mut book = LimitOrderBook::new();

        let order1 = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let order2 = Order {
            timestamp: 2000,
            id: 2,
            price: 10_100,
            quantity: 5,
            instrument: 0,
            side: Side::Buy,
        };

        book.add_order(order1.clone());
        book.add_order(order2.clone());

        let orders_100 = book.get_orders_at_price(&Side::Buy, 10_000);
        let orders_101 = book.get_orders_at_price(&Side::Buy, 10_100);

        assert!(orders_100.is_some());
        assert!(orders_101.is_some());
        assert_eq!(orders_100.unwrap().len(), 1);
        assert_eq!(orders_101.unwrap().len(), 1);
        assert_eq!(orders_100.unwrap()[0], order1);
        assert_eq!(orders_101.unwrap()[0], order2);
    }

    #[test]
    fn test_buy_matches_existing_sell() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        // Add a sell order first
        let sell_order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Sell,
        };
        book.add_order(sell_order.clone());

        // Try to match with a buy order at same price
        let buy_order = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let result = book.try_match(buy_order.clone(), &mut trades);

        // Should have exactly 1 trade
        assert_eq!(result.trade_count, 1);
        assert_eq!(result.remaining_quantity, 0);  // Fully filled

        let (matched_buy, matched_sell) = &trades[0];
        assert_eq!(matched_buy.id, buy_order.id);
        assert_eq!(matched_buy.quantity, 10);  // Full quantity traded
        assert_eq!(matched_sell.id, sell_order.id);
        assert_eq!(matched_sell.quantity, 10);

        // Verify sell order was removed from book
        assert!(book.get_orders_at_price(&Side::Sell, 10_000).is_none());
    }

    #[test]
    fn test_sell_matches_existing_buy() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        // Add a buy order first
        let buy_order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };
        book.add_order(buy_order.clone());

        // Try to match with a sell order at same price
        let sell_order = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Sell,
        };

        let result = book.try_match(sell_order.clone(), &mut trades);

        // Should have exactly 1 trade
        assert_eq!(result.trade_count, 1);
        assert_eq!(result.remaining_quantity, 0);  // Fully filled

        let (matched_buy, matched_sell) = &trades[0];
        assert_eq!(matched_buy.id, buy_order.id);
        assert_eq!(matched_buy.quantity, 10);
        assert_eq!(matched_sell.id, sell_order.id);
        assert_eq!(matched_sell.quantity, 10);

        // Verify buy order was removed from book
        assert!(book.get_orders_at_price(&Side::Buy, 10_000).is_none());
    }

    #[test]
    fn test_no_match_different_price() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        // Add a sell order at $104.00
        let sell_order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_400,
            quantity: 10,
            instrument: 0,
            side: Side::Sell,
        };
        book.add_order(sell_order.clone());

        // Try to match with a buy order at $100.00
        let buy_order = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let result = book.try_match(buy_order.clone(), &mut trades);

        // No trades should occur
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.remaining_quantity, 10);  // All quantity goes to book

        // Verify buy order was added to book
        let orders = book.get_orders_at_price(&Side::Buy, 10_000);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap()[0].id, buy_order.id);

        // Verify sell order still in book
        let orders = book.get_orders_at_price(&Side::Sell, 10_400);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap()[0].id, sell_order.id);
    }

    #[test]
    fn test_no_match_empty_book() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        let buy_order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let result = book.try_match(buy_order.clone(), &mut trades);

        // No trades should occur
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.remaining_quantity, 10);  // All quantity goes to book

        // Verify order was added to book
        let orders = book.get_orders_at_price(&Side::Buy, 10_000);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap()[0].id, buy_order.id);
    }

    #[test]
    fn test_partial_fill_buy_matches_multiple_sells() {
        let mut book = LimitOrderBook::new();

        // Add three sell orders at $100.00
        let sell_order1 = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 3,
            instrument: 0,
            side: Side::Sell,
        };
        let sell_order2 = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 2,
            instrument: 0,
            side: Side::Sell,
        };
        let sell_order3 = Order {
            timestamp: 3000,
            id: 3,
            price: 10_000,
            quantity: 4,
            instrument: 0,
            side: Side::Sell,
        };

        book.add_order(sell_order1.clone());
        book.add_order(sell_order2.clone());
        book.add_order(sell_order3.clone());

        // Try to buy 8 units - should match sell1 (3) + sell2 (2) + partial sell3 (3)
        let buy_order = Order {
            timestamp: 4000,
            id: 10,
            price: 10_000,
            quantity: 8,
            instrument: 0,
            side: Side::Buy,
        };

        let mut trades = Vec::new();
        let result = book.try_match(buy_order.clone(), &mut trades);

        // Should have 3 trades
        assert_eq!(result.trade_count, 3);

        // First trade: buy 3 units from sell1
        let (buy1, sell1) = &trades[0];
        assert_eq!(buy1.id, 10);
        assert_eq!(buy1.quantity, 3);
        assert_eq!(sell1.id, 1);
        assert_eq!(sell1.quantity, 3);

        // Second trade: buy 2 units from sell2
        let (buy2, sell2) = &trades[1];
        assert_eq!(buy2.id, 10);
        assert_eq!(buy2.quantity, 2);
        assert_eq!(sell2.id, 2);
        assert_eq!(sell2.quantity, 2);

        // Third trade: buy 3 units from sell3 (partial fill of sell3)
        let (buy3, sell3) = &trades[2];
        assert_eq!(buy3.id, 10);
        assert_eq!(buy3.quantity, 3);
        assert_eq!(sell3.id, 3);
        assert_eq!(sell3.quantity, 3);  // Only 3 out of 4 sold

        // All 8 units matched, so no remaining quantity
        assert_eq!(result.remaining_quantity, 0);

        // Verify sell3 is still in book with 1 unit remaining
        let orders = book.get_orders_at_price(&Side::Sell, 10_000);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0].id, 3);
        assert_eq!(orders.unwrap()[0].quantity, 1);  // 4 - 3 = 1 remaining

        // Verify buy order not in book (fully filled)
        let orders = book.get_orders_at_price(&Side::Buy, 10_000);
        assert!(orders.is_none());
    }

    #[test]
    fn test_partial_fill_sell_partially_matches_buy() {
        let mut book = LimitOrderBook::new();

        // Add a buy order with quantity 5
        let buy_order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 5,
            instrument: 0,
            side: Side::Buy,
        };
        book.add_order(buy_order.clone());

        // Sell order with quantity 8 - will partially match buy (5 units), leaving 3
        let sell_order = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 8,
            instrument: 0,
            side: Side::Sell,
        };

        let mut trades = Vec::new();
        let result = book.try_match(sell_order.clone(), &mut trades);

        // Should have 1 trade for 5 units
        assert_eq!(result.trade_count, 1);

        let (buy, sell) = &trades[0];
        assert_eq!(buy.id, 1);
        assert_eq!(buy.quantity, 5);  // Buy order fully consumed
        assert_eq!(sell.id, 2);
        assert_eq!(sell.quantity, 5);  // Only 5 out of 8 sold

        // 3 units remain unfilled
        assert_eq!(result.remaining_quantity, 3);

        // Verify buy order was fully consumed (removed from book)
        assert!(book.get_orders_at_price(&Side::Buy, 10_000).is_none());

        // Verify the remaining 3 units of sell order are in book
        let orders = book.get_orders_at_price(&Side::Sell, 10_000);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0].id, 2);
        assert_eq!(orders.unwrap()[0].quantity, 3);  // Remaining quantity
    }

    #[test]
    fn test_fifo_order_matching() {
        let mut book = LimitOrderBook::new();

        // Add two sell orders at same price
        let sell_order1 = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Sell,
        };
        let sell_order2 = Order {
            timestamp: 2000,
            id: 2,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Sell,
        };

        book.add_order(sell_order1.clone());
        book.add_order(sell_order2.clone());

        // Match with a buy order - should match oldest (sell_order1)
        let buy_order = Order {
            timestamp: 3000,
            id: 3,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let mut trades = Vec::new();
        let result = book.try_match(buy_order.clone(), &mut trades);

        // Should have exactly 1 trade
        assert_eq!(result.trade_count, 1);
        assert_eq!(result.remaining_quantity, 0);  // Fully filled

        let (matched_buy, matched_sell) = &trades[0];
        assert_eq!(matched_buy.id, buy_order.id);
        assert_eq!(matched_sell.id, sell_order1.id); // Should match the FIRST sell order (FIFO)

        // Verify only sell_order2 remains in book
        let orders = book.get_orders_at_price(&Side::Sell, 10_000);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0].id, sell_order2.id);
    }

    #[test]
    fn test_zero_quantity_order_rejected() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        let invalid_order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 0,  // Invalid!
            instrument: 0,
            side: Side::Buy,
        };

        let result = book.try_match(invalid_order, &mut trades);

        // Order should be rejected
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.remaining_quantity, 0);
        assert!(result.validation_error.is_some());

        // Verify order not added to book
        assert!(book.get_orders_at_price(&Side::Buy, 10_000).is_none());
    }

    #[test]
    fn test_zero_price_order_rejected() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        let invalid_order = Order {
            timestamp: 1000,
            id: 1,
            price: 0,  // Invalid!
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        let result = book.try_match(invalid_order, &mut trades);

        // Order should be rejected
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.remaining_quantity, 10);
        assert!(result.validation_error.is_some());

        // Verify order not added to book
        assert!(book.get_orders_at_price(&Side::Buy, 10_000).is_none());
    }

    #[test]
    fn test_circuit_breaker_rejects_out_of_range_price() {
        let mut book = LimitOrderBook::new();
        let mut trades = Vec::new();

        // Price below MIN_TICK (circuit breaker)
        let too_low = Order {
            timestamp: 1000,
            id: 1,
            price: 5_000, // Below MIN_TICK (10,000)
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };
        let result = book.try_match(too_low, &mut trades);
        assert_eq!(result.validation_error, Some(OrderValidationError::PriceOutOfRange));
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.remaining_quantity, 10);

        // Price above MAX_TICK (circuit breaker)
        let too_high = Order {
            timestamp: 2000,
            id: 2,
            price: 25_000, // Above MAX_TICK (20,000)
            quantity: 5,
            instrument: 0,
            side: Side::Sell,
        };
        let result = book.try_match(too_high, &mut trades);
        assert_eq!(result.validation_error, Some(OrderValidationError::PriceOutOfRange));
        assert_eq!(result.trade_count, 0);
        assert_eq!(result.remaining_quantity, 5);

        // Edge: exactly at MIN_TICK and MAX_TICK should be accepted
        let at_min = Order {
            timestamp: 3000,
            id: 3,
            price: MIN_TICK,
            quantity: 1,
            instrument: 0,
            side: Side::Buy,
        };
        let result = book.try_match(at_min, &mut trades);
        assert!(result.validation_error.is_none());

        let at_max = Order {
            timestamp: 4000,
            id: 4,
            price: MAX_TICK,
            quantity: 1,
            instrument: 0,
            side: Side::Sell,
        };
        let result = book.try_match(at_max, &mut trades);
        assert!(result.validation_error.is_none());
    }
}
