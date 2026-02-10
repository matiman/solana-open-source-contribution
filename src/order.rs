use crate::instrument::Instrument;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OrderValidationError {
    ZeroQuantity,
    InvalidPrice,
    PriceOutOfRange,
    InvalidInstrument,
}

impl std::fmt::Display for OrderValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroQuantity => write!(f, "zero quantity"),
            Self::InvalidPrice => write!(f, "invalid price"),
            Self::PriceOutOfRange => {
                write!(f, "price outside instrument's allowed tick range (circuit breaker)")
            }
            Self::InvalidInstrument => write!(f, "unknown instrument ID"),
        }
    }
}

impl std::error::Error for OrderValidationError {}

/// Order struct optimized for cache efficiency.
/// Size: 24 bytes (down from 40 bytes with u64 fields).
/// Field order is optimized for minimal padding with 8-byte alignment.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[repr(C)] // Prevent Rust from reordering fields
pub struct Order {
    pub timestamp: u64,           // Nanoseconds since Unix epoch — stamped at the gateway, not the matcher
    pub id: u32,                  // Order ID (max 4.2B orders)
    pub price: u32,               // Price in ticks (1 tick = $0.01). Max 4.2B ticks = $42M
    pub quantity: u32,            // Quantity (max 4.2B units)
    pub instrument: Instrument,   // Instrument ID (e.g., BTCUSDC, SOLUSDC, etc.)
    pub side: Side,               // Buy or Sell
    // 2 bytes padding here to reach 24-byte alignment
}

impl Order {
    /// Smart constructor - creates a new Order with validation
    /// Returns Ok(Order) if valid, Err(OrderValidationError) otherwise
    /// This prevents invalid orders from being created in the first place
    pub fn new(
        id: u32,
        instrument: Instrument,
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

        if !self.instrument.is_valid() {
            return Err(OrderValidationError::InvalidInstrument);
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
            instrument: Instrument::BTCUSDC,
            side: Side::Buy,
        };

        assert_eq!(order.id, 1);
        assert_eq!(order.instrument, Instrument::BTCUSDC);
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
            instrument: Instrument::BTCUSDC,
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
            instrument: Instrument::BTCUSDC,
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
            instrument: Instrument::BTCUSDC,
            side: Side::Buy,
        };

        assert_eq!(order.validate(), Err(OrderValidationError::InvalidPrice));
    }

    #[test]
    fn test_smart_constructor_valid_order() {
        let result = Order::new(1, Instrument::BTCUSDC, Side::Buy, 10_000, 10, 1000);
        assert!(result.is_ok());

        let order = result.unwrap();
        assert_eq!(order.id, 1);
        assert_eq!(order.instrument, Instrument::BTCUSDC);
        assert_eq!(order.side, Side::Buy);
        assert_eq!(order.price, 10_000);
        assert_eq!(order.quantity, 10);
        assert_eq!(order.timestamp, 1000);
    }

    #[test]
    fn test_smart_constructor_zero_quantity() {
        let result = Order::new(1, Instrument::BTCUSDC, Side::Buy, 10_000, 0, 1000);
        assert_eq!(result, Err(OrderValidationError::ZeroQuantity));
    }

    #[test]
    fn test_smart_constructor_zero_price() {
        let result = Order::new(1, Instrument::BTCUSDC, Side::Buy, 0, 10, 1000);
        assert_eq!(result, Err(OrderValidationError::InvalidPrice));
    }

    #[test]
    fn test_invalid_instrument_rejected() {
        let order = Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: Instrument::new(99).unwrap_or(Instrument::BTCUSDC),
            side: Side::Buy,
        };
        // Instrument::new(99) returns None, so we need a different approach to construct an invalid one.
        // Since the inner field is private, invalid instruments only come from deserialization.
        let encoded = bincode::serialize(&Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 10,
            instrument: Instrument::BTCUSDC,
            side: Side::Buy,
        })
        .unwrap();
        // Patch the instrument byte (offset: 8+4+4+4 = 20) to an invalid value
        let mut patched = encoded.clone();
        patched[20] = 99;
        let bad_order: Order = bincode::deserialize(&patched).unwrap();
        assert_eq!(
            bad_order.validate(),
            Err(OrderValidationError::InvalidInstrument)
        );
        // Suppress unused variable warning
        let _ = order;
    }

    #[test]
    fn test_bincode_roundtrip() {
        let order = Order {
            timestamp: 1234567890,
            id: 42,
            price: 15_000,
            quantity: 7,
            instrument: Instrument::ETHUSDC,
            side: Side::Sell,
        };
        let encoded = bincode::serialize(&order).unwrap();
        let decoded: Order = bincode::deserialize(&encoded).unwrap();
        assert_eq!(order, decoded);
    }

    #[test]
    fn test_side_bincode_roundtrip() {
        let buy_encoded = bincode::serialize(&Side::Buy).unwrap();
        let sell_encoded = bincode::serialize(&Side::Sell).unwrap();
        assert_eq!(bincode::deserialize::<Side>(&buy_encoded).unwrap(), Side::Buy);
        assert_eq!(bincode::deserialize::<Side>(&sell_encoded).unwrap(), Side::Sell);
    }
}
