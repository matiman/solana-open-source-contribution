use crate::order::{Order, Side};
use std::collections::VecDeque;

const MIN_PRICE: u64 = 100;
const PRICE_LEVELS: usize = 5;

pub struct OrderBook {
    // Fixed array for price levels [100, 101, 102, 103, 104]. FIFO matching.
    // Index = price - MIN_PRICE
    buys: [Option<VecDeque<Order>>; PRICE_LEVELS],
    sells: [Option<VecDeque<Order>>; PRICE_LEVELS],
}


impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            buys: [None, None, None, None, None],
            sells: [None, None, None, None, None],
        }
    }

    #[inline]
    fn price_to_index(price: u64) -> usize {
        (price - MIN_PRICE) as usize
    }

    pub fn add_order(&mut self, order: Order) {
        let book = match order.side {
            Side::Buy => &mut self.buys,
            Side::Sell => &mut self.sells,
        };

        let idx = Self::price_to_index(order.price);
        book[idx]
            .get_or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn get_orders_at_price(&self, side: &Side, price: u64) -> Option<&VecDeque<Order>> {
        let book = match side {
            Side::Buy => &self.buys,
            Side::Sell => &self.sells,
        };

        let idx = Self::price_to_index(price);
        book[idx].as_ref()
    }

    pub fn try_match(&mut self, order: Order) -> Option<(Order, Order)> {
        let idx = Self::price_to_index(order.price);

        match order.side {
            Side::Buy => {
                // Buy order: look for matching sell at same price
                if let Some(sell_queue) = self.sells[idx].as_mut()
                    && let Some(matched_sell) = sell_queue.pop_front()
                {
                    // Clean up empty queue
                    if sell_queue.is_empty() {
                        self.sells[idx] = None;
                    }
                    return Some((order, matched_sell));
                }
                // No match: add buy order to book
                self.add_order(order);
                None
            }
            Side::Sell => {
                // Sell order: look for matching buy at same price
                if let Some(buy_queue) = self.buys[idx].as_mut()
                    && let Some(matched_buy) = buy_queue.pop_front()
                {
                    // Clean up empty queue
                    if buy_queue.is_empty() {
                        self.buys[idx] = None;
                    }
                    return Some((matched_buy, order));
                }
                // No match: add sell order to book
                self.add_order(order);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_buy_order() {
        let mut book = OrderBook::new();
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };

        book.add_order(order.clone());

        let orders = book.get_orders_at_price(&Side::Buy, 100);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0], order);
    }

    #[test]
    fn test_add_sell_order() {
        let mut book = OrderBook::new();
        let order = Order {
            id: 2,
            side: Side::Sell,
            price: 104,
            quantity: 5,
            timestamp: 2000,
        };

        book.add_order(order.clone());

        let orders = book.get_orders_at_price(&Side::Sell, 104);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0], order);
    }

    #[test]
    fn test_multiple_orders_same_price() {
        let mut book = OrderBook::new();

        let order1 = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };

        let order2 = Order {
            id: 2,
            side: Side::Buy,
            price: 100,
            quantity: 5,
            timestamp: 2000,
        };

        book.add_order(order1.clone());
        book.add_order(order2.clone());

        let orders = book.get_orders_at_price(&Side::Buy, 100);
        assert!(orders.is_some());
        let orders_vec = orders.unwrap();
        assert_eq!(orders_vec.len(), 2);
        assert_eq!(orders_vec[0], order1);
        assert_eq!(orders_vec[1], order2);
    }

    #[test]
    fn test_orders_different_prices() {
        let mut book = OrderBook::new();

        let order1 = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };

        let order2 = Order {
            id: 2,
            side: Side::Buy,
            price: 101,
            quantity: 5,
            timestamp: 2000,
        };

        book.add_order(order1.clone());
        book.add_order(order2.clone());

        let orders_100 = book.get_orders_at_price(&Side::Buy, 100);
        let orders_101 = book.get_orders_at_price(&Side::Buy, 101);

        assert!(orders_100.is_some());
        assert!(orders_101.is_some());
        assert_eq!(orders_100.unwrap().len(), 1);
        assert_eq!(orders_101.unwrap().len(), 1);
        assert_eq!(orders_100.unwrap()[0], order1);
        assert_eq!(orders_101.unwrap()[0], order2);
    }

    #[test]
    fn test_buy_matches_existing_sell() {
        let mut book = OrderBook::new();

        // Add a sell order first
        let sell_order = Order {
            id: 1,
            side: Side::Sell,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };
        book.add_order(sell_order.clone());

        // Try to match with a buy order at same price
        let buy_order = Order {
            id: 2,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 2000,
        };

        let result = book.try_match(buy_order.clone());
        assert!(result.is_some());
        let (matched_buy, matched_sell) = result.unwrap();
        assert_eq!(matched_buy, buy_order);
        assert_eq!(matched_sell, sell_order);

        // Verify sell order was removed from book
        assert!(book.get_orders_at_price(&Side::Sell, 100).is_none());
    }

    #[test]
    fn test_sell_matches_existing_buy() {
        let mut book = OrderBook::new();

        // Add a buy order first
        let buy_order = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };
        book.add_order(buy_order.clone());

        // Try to match with a sell order at same price
        let sell_order = Order {
            id: 2,
            side: Side::Sell,
            price: 100,
            quantity: 10,
            timestamp: 2000,
        };

        let result = book.try_match(sell_order.clone());
        assert!(result.is_some());
        let (matched_buy, matched_sell) = result.unwrap();
        assert_eq!(matched_buy, buy_order);
        assert_eq!(matched_sell, sell_order);

        // Verify buy order was removed from book
        assert!(book.get_orders_at_price(&Side::Buy, 100).is_none());
    }

    #[test]
    fn test_no_match_different_price() {
        let mut book = OrderBook::new();

        // Add a sell order at price 104
        let sell_order = Order {
            id: 1,
            side: Side::Sell,
            price: 104,
            quantity: 10,
            timestamp: 1000,
        };
        book.add_order(sell_order.clone());

        // Try to match with a buy order at price 100
        let buy_order = Order {
            id: 2,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 2000,
        };

        let result = book.try_match(buy_order.clone());
        assert!(result.is_none());

        // Verify buy order was added to book
        let orders = book.get_orders_at_price(&Side::Buy, 100);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap()[0], buy_order);

        // Verify sell order still in book
        let orders = book.get_orders_at_price(&Side::Sell, 104);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap()[0], sell_order);
    }

    #[test]
    fn test_no_match_empty_book() {
        let mut book = OrderBook::new();

        let buy_order = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };

        let result = book.try_match(buy_order.clone());
        assert!(result.is_none());

        // Verify order was added to book
        let orders = book.get_orders_at_price(&Side::Buy, 100);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap()[0], buy_order);
    }

    #[test]
    fn test_fifo_order_matching() {
        let mut book = OrderBook::new();

        // Add two sell orders at same price
        let sell_order1 = Order {
            id: 1,
            side: Side::Sell,
            price: 100,
            quantity: 10,
            timestamp: 1000,
        };
        let sell_order2 = Order {
            id: 2,
            side: Side::Sell,
            price: 100,
            quantity: 10,
            timestamp: 2000,
        };

        book.add_order(sell_order1.clone());
        book.add_order(sell_order2.clone());

        // Match with a buy order - should match oldest (sell_order1)
        let buy_order = Order {
            id: 3,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 3000,
        };

        let result = book.try_match(buy_order.clone());
        assert!(result.is_some());
        let (matched_buy, matched_sell) = result.unwrap();
        assert_eq!(matched_buy, buy_order);
        assert_eq!(matched_sell, sell_order1); // Should match the FIRST sell order

        // Verify only sell_order2 remains in book
        let orders = book.get_orders_at_price(&Side::Sell, 100);
        assert!(orders.is_some());
        assert_eq!(orders.unwrap().len(), 1);
        assert_eq!(orders.unwrap()[0], sell_order2);
    }
}
