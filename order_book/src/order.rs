#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,
    pub quantity: u64,
    pub timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 100,
            quantity: 10,
            timestamp: 1234567890,
        };

        assert_eq!(order.id, 1);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 100);
        assert_eq!(order.quantity, 10);
        assert_eq!(order.timestamp, 1234567890);
    }

    #[test]
    fn test_side_equality() {
        let buy1 = Side::Buy;
        let buy2 = Side::Buy;
        let sell = Side::Sell;

        assert_eq!(buy1, buy2);
        assert_ne!(buy1, sell);
    }
}
