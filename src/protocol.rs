use crate::order::Order;
use std::io::{self, Read, Write};

/// Maximum number of orders per batch (1024 × 24 bytes = 24KB, fits L1 cache)
pub const MAX_BATCH_SIZE: usize = 1024;

/// Maximum payload size in bytes (generous upper bound for bincode encoding)
const MAX_PAYLOAD_SIZE: u32 = MAX_BATCH_SIZE as u32 * 64;

/// Write a batch of orders to a writer (TCP stream).
/// Format: [payload_len: u32 LE][bincode-encoded Vec<Order>]
pub fn write_batch(writer: &mut impl Write, batch: &[Order]) -> io::Result<()> {
    let encoded = bincode::serialize(batch)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let len = encoded.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&encoded)?;
    Ok(())
}

/// Write a shutdown sentinel (payload_len = 0).
pub fn write_shutdown(writer: &mut impl Write) -> io::Result<()> {
    writer.write_all(&0u32.to_le_bytes())
}

/// Read a batch of orders from a reader (TCP stream).
/// Returns `None` on shutdown sentinel (payload_len = 0) or EOF.
pub fn read_batch(reader: &mut impl Read) -> io::Result<Option<Vec<Order>>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf) {
        Ok(()) => {}
        Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e),
    }

    let payload_len = u32::from_le_bytes(len_buf);

    // Shutdown sentinel
    if payload_len == 0 {
        return Ok(None);
    }

    // Reject oversized payloads
    if payload_len > MAX_PAYLOAD_SIZE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "payload size {} exceeds maximum {}",
                payload_len, MAX_PAYLOAD_SIZE
            ),
        ));
    }

    let mut payload = vec![0u8; payload_len as usize];
    reader.read_exact(&mut payload)?;

    let orders: Vec<Order> = bincode::deserialize(&payload)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    Ok(Some(orders))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instrument::Instrument;
    use crate::order::Side;
    use std::io::Cursor;

    #[test]
    fn test_roundtrip_batch() {
        let orders = vec![
            Order {
                timestamp: 1000,
                id: 1,
                price: 15_000,
                quantity: 5,
                instrument: Instrument::BTCUSDC,
                side: Side::Buy,
            },
            Order {
                timestamp: 2000,
                id: 2,
                price: 16_000,
                quantity: 3,
                instrument: Instrument::SOLUSDC,
                side: Side::Sell,
            },
        ];

        let mut buf = Vec::new();
        write_batch(&mut buf, &orders).unwrap();

        let mut cursor = Cursor::new(&buf);
        let result = read_batch(&mut cursor).unwrap().unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], orders[0]);
        assert_eq!(result[1], orders[1]);
    }

    #[test]
    fn test_shutdown_sentinel() {
        let mut buf = Vec::new();
        write_shutdown(&mut buf).unwrap();

        let mut cursor = Cursor::new(&buf);
        let result = read_batch(&mut cursor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_eof_returns_none() {
        let buf: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&buf);
        let result = read_batch(&mut cursor).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_oversized_payload_rejected() {
        let mut buf = Vec::new();
        let huge_len = MAX_PAYLOAD_SIZE + 1;
        buf.extend_from_slice(&huge_len.to_le_bytes());

        let mut cursor = Cursor::new(&buf);
        let result = read_batch(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_batch_roundtrip() {
        let orders: Vec<Order> = Vec::new();

        let mut buf = Vec::new();
        write_batch(&mut buf, &orders).unwrap();

        let mut cursor = Cursor::new(&buf);
        let result = read_batch(&mut cursor).unwrap().unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_multiple_batches_sequential() {
        let batch1 = vec![Order {
            timestamp: 1000,
            id: 1,
            price: 10_000,
            quantity: 1,
            instrument: Instrument::BTCUSDC,
            side: Side::Buy,
        }];
        let batch2 = vec![Order {
            timestamp: 2000,
            id: 2,
            price: 11_000,
            quantity: 2,
            instrument: Instrument::HYPEUSDC,
            side: Side::Sell,
        }];

        let mut buf = Vec::new();
        write_batch(&mut buf, &batch1).unwrap();
        write_batch(&mut buf, &batch2).unwrap();
        write_shutdown(&mut buf).unwrap();

        let mut cursor = Cursor::new(&buf);
        let r1 = read_batch(&mut cursor).unwrap().unwrap();
        let r2 = read_batch(&mut cursor).unwrap().unwrap();
        let r3 = read_batch(&mut cursor).unwrap();

        assert_eq!(r1.len(), 1);
        assert_eq!(r1[0].id, 1);
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0].id, 2);
        assert!(r3.is_none()); // shutdown
    }
}
