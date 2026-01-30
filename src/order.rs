#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderValidationError {
    ZeroQuantity,
    InvalidPrice,
    PriceOutOfRange,
}

impl std::fmt::Display for OrderValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroQuantity => write!(f, "zero quantity"),
            Self::InvalidPrice => write!(f, "invalid price"),
            Self::PriceOutOfRange => {
                write!(f, "price outside instrument's allowed tick range (circuit breaker)")
            }
        }
    }
}

impl std::error::Error for OrderValidationError {}

#[derive(Debug, Clone, PartialEq)]
pub struct Order {
    pub id: u64,
    pub side: Side,
    pub price: u64,     // Price in ticks (1 tick = $0.01). Integer — no floats.
    pub quantity: u64,
    pub timestamp: u64, // Nanoseconds since Unix epoch — stamped at the gateway, not the matcher
}

impl Order {
    /// Smart constructor - creates a new Order with validation
    /// Returns Ok(Order) if valid, Err(OrderValidationError) otherwise
    /// This prevents invalid orders from being created in the first place
    pub fn new(
        id: u64,
        side: Side,
        price: u64,
        quantity: u64,
        timestamp: u64,
    ) -> Result<Self, OrderValidationError> {
        if quantity == 0 {
            return Err(OrderValidationError::ZeroQuantity);
        }

        if price == 0 {
            return Err(OrderValidationError::InvalidPrice);
        }

        Ok(Order {
            id,
            side,
            price,
            quantity,
            timestamp,
        })
    }

    /// Validates an existing order
    /// Returns Ok(()) if valid, Err(OrderValidationError) otherwise
    pub fn validate(&self) -> Result<(), OrderValidationError> {
        if self.quantity == 0 {
            return Err(OrderValidationError::ZeroQuantity);
        }

        if self.price == 0 {
            return Err(OrderValidationError::InvalidPrice);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_creation() {
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 10_050, // $100.50 in ticks
            quantity: 10,
            timestamp: 1234567890,
        };

        assert_eq!(order.id, 1);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 10_050);
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

    #[test]
    fn test_valid_order() {
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 10_000,
            quantity: 10,
            timestamp: 1000,
        };

        assert!(order.validate().is_ok());
    }

    #[test]
    fn test_zero_quantity_order() {
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 10_000,
            quantity: 0,
            timestamp: 1000,
        };

        assert_eq!(order.validate(), Err(OrderValidationError::ZeroQuantity));
    }

    #[test]
    fn test_zero_price_order() {
        let order = Order {
            id: 1,
            side: Side::Buy,
            price: 0,
            quantity: 10,
            timestamp: 1000,
        };

        assert_eq!(order.validate(), Err(OrderValidationError::InvalidPrice));
    }

    #[test]
    fn test_smart_constructor_valid_order() {
        let result = Order::new(1, Side::Buy, 10_000, 10, 1000);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.id, 1);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 10_000);
        assert_eq!(order.quantity, 10);
        assert_eq!(order.timestamp, 1000);
    }

    #[test]
    fn test_smart_constructor_zero_quantity() {
        let result = Order::new(1, Side::Buy, 10_000, 0, 1000);
        assert_eq!(result, Err(OrderValidationError::ZeroQuantity));
    }

    #[test]
    fn test_smart_constructor_zero_price() {
        let result = Order::new(1, Side::Buy, 0, 10, 1000);
        assert_eq!(result, Err(OrderValidationError::InvalidPrice));
    }
}
