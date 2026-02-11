/// Instrument newtype — prevents treating instruments as plain numbers,
/// restricts operations to what we provide, and makes future type changes easier.
///
/// `#[repr(transparent)]` guarantees same layout as `u8` — zero overhead,
/// Order struct stays 24 bytes, and bincode serializes as raw `u8`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(transparent)]
pub struct Instrument(u8);

impl Instrument {
    pub const BTCUSDC: Instrument = Instrument(0);
    pub const SOLUSDC: Instrument = Instrument(1);
    pub const ETHUSDC: Instrument = Instrument(2);
    pub const HYPEUSDC: Instrument = Instrument(3);

    /// Create an Instrument from a raw u8. Returns `None` if invalid.
    pub fn new(id: u8) -> Option<Instrument> {
        if (id as usize) < NUM_INSTRUMENTS {
            Some(Instrument(id))
        } else {
            None
        }
    }

    #[inline(always)]
    pub fn as_usize(self) -> usize {
        self.0 as usize
    }

    #[inline(always)]
    pub fn as_u8(self) -> u8 {
        self.0
    }

    pub fn is_valid(self) -> bool {
        (self.0 as usize) < NUM_INSTRUMENTS
    }
}

impl std::fmt::Display for Instrument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::BTCUSDC => write!(f, "BTCUSDC"),
            Self::SOLUSDC => write!(f, "SOLUSDC"),
            Self::ETHUSDC => write!(f, "ETHUSDC"),
            Self::HYPEUSDC => write!(f, "HYPEUSDC"),
            _ => write!(f, "UNKNOWN"),
        }
    }
}

pub const NUM_INSTRUMENTS: usize = 4;

/// All instrument IDs for iteration
pub const ALL_INSTRUMENTS: [Instrument; NUM_INSTRUMENTS] = [
    Instrument::BTCUSDC,
    Instrument::SOLUSDC,
    Instrument::ETHUSDC,
    Instrument::HYPEUSDC,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instrument_display() {
        assert_eq!(format!("{}", Instrument::BTCUSDC), "BTCUSDC");
        assert_eq!(format!("{}", Instrument::SOLUSDC), "SOLUSDC");
        assert_eq!(format!("{}", Instrument::ETHUSDC), "ETHUSDC");
        assert_eq!(format!("{}", Instrument::HYPEUSDC), "HYPEUSDC");
        assert_eq!(format!("{}", Instrument(99)), "UNKNOWN");
    }

    #[test]
    fn test_instrument_new() {
        assert_eq!(Instrument::new(0), Some(Instrument::BTCUSDC));
        assert_eq!(Instrument::new(3), Some(Instrument::HYPEUSDC));
        assert_eq!(Instrument::new(4), None);
        assert_eq!(Instrument::new(99), None);
    }

    #[test]
    fn test_instrument_is_valid() {
        assert!(Instrument::BTCUSDC.is_valid());
        assert!(Instrument::HYPEUSDC.is_valid());
        assert!(!Instrument(4).is_valid());
        assert!(!Instrument(99).is_valid());
    }

    #[test]
    fn test_instrument_as_usize() {
        assert_eq!(Instrument::BTCUSDC.as_usize(), 0);
        assert_eq!(Instrument::SOLUSDC.as_usize(), 1);
        assert_eq!(Instrument::ETHUSDC.as_usize(), 2);
        assert_eq!(Instrument::HYPEUSDC.as_usize(), 3);
    }

    #[test]
    fn test_all_instruments() {
        assert_eq!(ALL_INSTRUMENTS.len(), NUM_INSTRUMENTS);
        assert_eq!(ALL_INSTRUMENTS[0], Instrument::BTCUSDC);
        assert_eq!(ALL_INSTRUMENTS[3], Instrument::HYPEUSDC);
    }

    #[test]
    fn test_bincode_roundtrip() {
        // Instrument serializes as raw u8 thanks to #[repr(transparent)]
        let inst = Instrument::ETHUSDC;
        let encoded = bincode::serialize(&inst).unwrap();
        let decoded: Instrument = bincode::deserialize(&encoded).unwrap();
        assert_eq!(inst, decoded);

        // Raw u8 value 2 should deserialize to ETHUSDC
        let raw = bincode::serialize(&2u8).unwrap();
        let decoded: Instrument = bincode::deserialize(&raw).unwrap();
        assert_eq!(decoded, Instrument::ETHUSDC);
    }
}
