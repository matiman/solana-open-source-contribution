/// Instrument identifiers for trading pairs.
/// Using u8 for compact representation in Order struct.

pub const BTCUSDC: u8 = 0;
pub const SOLUSDC: u8 = 1;
pub const ETHUSDC: u8 = 2;
pub const HYPEUSDC: u8 = 3;

pub const NUM_INSTRUMENTS: usize = 4;

/// All instrument IDs for iteration
pub const ALL_INSTRUMENTS: [u8; NUM_INSTRUMENTS] = [BTCUSDC, SOLUSDC, ETHUSDC, HYPEUSDC];

/// Get human-readable name for an instrument
pub fn name(instrument: u8) -> &'static str {
    match instrument {
        BTCUSDC => "BTCUSDC",
        SOLUSDC => "SOLUSDC",
        ETHUSDC => "ETHUSDC",
        HYPEUSDC => "HYPEUSDC",
        _ => "UNKNOWN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_names() {
        assert_eq!(name(BTCUSDC), "BTCUSDC");
        assert_eq!(name(SOLUSDC), "SOLUSDC");
        assert_eq!(name(ETHUSDC), "ETHUSDC");
        assert_eq!(name(HYPEUSDC), "HYPEUSDC");
        assert_eq!(name(99), "UNKNOWN");
    }

    #[test]
    fn test_all_instruments() {
        assert_eq!(ALL_INSTRUMENTS.len(), NUM_INSTRUMENTS);
        assert_eq!(ALL_INSTRUMENTS[0], BTCUSDC);
        assert_eq!(ALL_INSTRUMENTS[3], HYPEUSDC);
    }
}
