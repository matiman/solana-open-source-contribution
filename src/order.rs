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

/// Order struct optimized for cache efficiency.
/// Size: 24 bytes (down from 40 bytes with u64 fields).
/// Field order is optimized for minimal padding with 8-byte alignment.
#[derive(Debug, Clone, PartialEq)]
#[repr(C)] // Prevent Rust from reordering fields
pub struct Order {
    pub timestamp: u64, // Nanoseconds since Unix epoch — stamped at the gateway, not the matcher
    pub id: u32,        // Order ID (max 4.2B orders)
    pub price: u32,     // Price in ticks (1 tick = $0.01). Max 4.2B ticks = $42M
    pub quantity: u32,  // Quantity (max 4.2B units)
    pub instrument: u8, // Instrument ID (e.g., BTCUSDC=0, SOLUSDC=1, etc.)
    pub side: Side,     // Buy or Sell
    // 2 bytes padding here to reach 24-byte alignment
}

impl Order {
    /// Smart constructor - creates a new Order with validation
    /// Returns Ok(Order) if valid, Err(OrderValidationError) otherwise
    /// This prevents invalid orders from being created in the first place
    pub fn new(
        id: u32,
        instrument: u8,
        side: Side,
        price: u32,
        quantity: u32,
        timestamp: u64,
    ) -> Result<Self, OrderValidationError> {
        if quantity == 0 {
            return Err(OrderValidationError::ZeroQuantity);
        }

        if price == 0 {
            return Err(OrderValidationError::InvalidPrice);
        }

        Ok(Order {
            timestamp,
            id,
            price,
            quantity,
            instrument,
            side,
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
            timestamp: 1234567890,
            id: 1,
            price: 10_050, // $100.50 in ticks
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        assert_eq!(order.id, 1);
        assert_eq!(order.instrument, 0);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 10_050);
        assert_eq!(order.quantity, 10);
        assert_eq!(order.timestamp, 1234567890);
    }

    #[test]
    fn test_order_size() {
        // Verify Order struct is 24 bytes (optimized from 40 bytes)
        assert_eq!(std::mem::size_of::<Order>(), 24);
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
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        assert!(order.validate().is_ok());
    }

    #[test]
    fn test_zero_quantity_order() {
        let order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 0,
            instrument: 0,
            side: Side::Buy,
        };

        assert_eq!(order.validate(), Err(OrderValidationError::ZeroQuantity));
    }

    #[test]
    fn test_zero_price_order() {
        let order = Order {
            timestamp: 1000,
            id: 1,
            price: 0,
            quantity: 10,
            instrument: 0,
            side: Side::Buy,
        };

        assert_eq!(order.validate(), Err(OrderValidationError::InvalidPrice));
    }

    #[test]
    fn test_smart_constructor_valid_order() {
        let result = Order::new(1, 0, Side::Buy, 10_000, 10, 1000);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.id, 1);
        assert_eq!(order.instrument, 0);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 10_000);
        assert_eq!(order.quantity, 10);
        assert_eq!(order.timestamp, 1000);
    }

    #[test]
    fn test_smart_constructor_zero_quantity() {
        let result = Order::new(1, 0, Side::Buy, 10_000, 0, 1000);
        assert_eq!(result, Err(OrderValidationError::ZeroQuantity));
    }

    #[test]
    fn test_smart_constructor_zero_price() {
        let result = Order::new(1, 0, Side::Buy, 0, 10, 1000);
        assert_eq!(result, Err(OrderValidationError::InvalidPrice));
    }
}
