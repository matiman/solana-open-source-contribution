use crate::order::{Order, Side};
use std::collections::{HashMap, VecDeque};

#[allow(dead_code)]
pub struct OrderBook {
    buys: HashMap<u64, VecDeque<Order>>,
    sells: HashMap<u64, VecDeque<Order>>,
}

#[allow(dead_code)]
impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            buys: HashMap::new(),
            sells: HashMap::new(),
        }
    }

    pub fn add_order(&mut self, order: Order) {
        let book = match order.side {
            Side::Buy => &mut self.buys,
            Side::Sell => &mut self.sells,
        };

        book.entry(order.price)
            .or_insert_with(VecDeque::new)
            .push_back(order);
    }

    pub fn get_orders_at_price(&self, side: &Side, price: u64) -> Option<&VecDeque<Order>> {
        let book = match side {
            Side::Buy => &self.buys,
            Side::Sell => &self.sells,
        };

        book.get(&price)
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
            price: 105,
            quantity: 5,
            timestamp: 2000,
        };

        book.add_order(order.clone());

        let orders = book.get_orders_at_price(&Side::Sell, 105);
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
}
