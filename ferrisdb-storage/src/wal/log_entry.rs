use ferrisdb_core::{Key, Value, Operation, Timestamp, Result, Error};
use bytes::{Buf, BufMut, BytesMut};
use crc32fast::Hasher;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WALEntry {
    pub timestamp: Timestamp,
    pub operation: Operation,
    pub key: Key,
    pub value: Value,
}

impl WALEntry {
    pub fn new_put(key: Key, value: Value, timestamp: Timestamp) -> Self {
        Self {
            timestamp,
            operation: Operation::Put,
            key,
            value,
        }
    }
    
    pub fn new_delete(key: Key, timestamp: Timestamp) -> Self {
        Self {
            timestamp,
            operation: Operation::Delete,
            key,
            value: Vec::new(),
        }
    }
    
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        
        // Reserve space for length and checksum
        buf.put_u32_le(0); // length placeholder
        buf.put_u32_le(0); // checksum placeholder
        
        // Encode entry data
        buf.put_u64_le(self.timestamp);
        buf.put_u8(match self.operation {
            Operation::Put => 1,
            Operation::Delete => 2,
        });
        
        buf.put_u32_le(self.key.len() as u32);
        buf.put_slice(&self.key);
        
        buf.put_u32_le(self.value.len() as u32);
        buf.put_slice(&self.value);
        
        // Calculate and set length (excluding length field itself)
        let total_len = buf.len() - 4;
        buf[0..4].copy_from_slice(&(total_len as u32).to_le_bytes());
        
        // Calculate and set checksum (excluding length and checksum fields)
        let mut hasher = Hasher::new();
        hasher.update(&buf[8..]);
        let checksum = hasher.finalize();
        buf[4..8].copy_from_slice(&checksum.to_le_bytes());
        
        buf.to_vec()
    }
    
    pub fn decode(data: &[u8]) -> Result<Self> {
        if data.len() < 8 {
            return Err(Error::Corruption("WAL entry too small".to_string()));
        }
        
        let mut cursor = &data[..];
        
        // Read and verify length
        let length = cursor.get_u32_le() as usize;
        if data.len() != length + 4 {
            return Err(Error::Corruption("WAL entry length mismatch".to_string()));
        }
        
        // Read and verify checksum
        let expected_checksum = cursor.get_u32_le();
        let mut hasher = Hasher::new();
        hasher.update(&data[8..]);
        let actual_checksum = hasher.finalize();
        
        if expected_checksum != actual_checksum {
            return Err(Error::Corruption("WAL entry checksum mismatch".to_string()));
        }
        
        // Decode entry data
        let timestamp = cursor.get_u64_le();
        let operation = match cursor.get_u8() {
            1 => Operation::Put,
            2 => Operation::Delete,
            _ => return Err(Error::Corruption("Invalid operation type".to_string())),
        };
        
        let key_len = cursor.get_u32_le() as usize;
        if cursor.len() < key_len {
            return Err(Error::Corruption("Key length exceeds data".to_string()));
        }
        let key = cursor[..key_len].to_vec();
        cursor.advance(key_len);
        
        let value_len = cursor.get_u32_le() as usize;
        if cursor.len() < value_len {
            return Err(Error::Corruption("Value length exceeds data".to_string()));
        }
        let value = cursor[..value_len].to_vec();
        
        Ok(Self {
            timestamp,
            operation,
            key,
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encode_decode_put() {
        let entry = WALEntry::new_put(
            b"test_key".to_vec(),
            b"test_value".to_vec(),
            12345,
        );
        
        let encoded = entry.encode();
        let decoded = WALEntry::decode(&encoded).unwrap();
        
        assert_eq!(entry, decoded);
    }
    
    #[test]
    fn test_encode_decode_delete() {
        let entry = WALEntry::new_delete(
            b"test_key".to_vec(),
            12345,
        );
        
        let encoded = entry.encode();
        let decoded = WALEntry::decode(&encoded).unwrap();
        
        assert_eq!(entry, decoded);
    }
    
    #[test]
    fn test_corruption_detection() {
        let entry = WALEntry::new_put(
            b"test_key".to_vec(),
            b"test_value".to_vec(),
            12345,
        );
        
        let mut encoded = entry.encode();
        // Corrupt the data
        encoded[20] ^= 0xFF;
        
        let result = WALEntry::decode(&encoded);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), Error::Corruption(_)));
    }
}