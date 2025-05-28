use super::skiplist::SkipList;
use ferrisdb_core::{Key, Value, Operation, Timestamp, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct MemTable {
    skiplist: Arc<SkipList>,
    size: AtomicUsize,
    size_limit: usize,
}

impl MemTable {
    pub fn new(size_limit: usize) -> Self {
        Self {
            skiplist: Arc::new(SkipList::new()),
            size: AtomicUsize::new(0),
            size_limit,
        }
    }
    
    pub fn put(&self, key: Key, value: Value, timestamp: Timestamp) -> Result<()> {
        let entry_size = key.len() + value.len() + 16; // Approximate size
        
        if self.would_exceed_limit(entry_size) {
            return Err(ferrisdb_core::Error::StorageEngine(
                "MemTable size limit would be exceeded".to_string()
            ));
        }
        
        self.skiplist.insert(key, value, timestamp, Operation::Put);
        self.size.fetch_add(entry_size, Ordering::Relaxed);
        Ok(())
    }
    
    pub fn delete(&self, key: Key, timestamp: Timestamp) -> Result<()> {
        let entry_size = key.len() + 16; // Approximate size
        
        if self.would_exceed_limit(entry_size) {
            return Err(ferrisdb_core::Error::StorageEngine(
                "MemTable size limit would be exceeded".to_string()
            ));
        }
        
        self.skiplist.insert(key, Vec::new(), timestamp, Operation::Delete);
        self.size.fetch_add(entry_size, Ordering::Relaxed);
        Ok(())
    }
    
    pub fn get(&self, key: &[u8], timestamp: Timestamp) -> Option<(Value, Operation)> {
        self.skiplist.get(key, timestamp)
    }
    
    pub fn scan(&self, start_key: &[u8], end_key: &[u8], timestamp: Timestamp) -> Vec<(Key, Value)> {
        self.skiplist.scan(start_key, end_key, timestamp)
    }
    
    pub fn approximate_size(&self) -> usize {
        self.size.load(Ordering::Relaxed)
    }
    
    pub fn is_full(&self) -> bool {
        self.approximate_size() >= self.size_limit
    }
    
    fn would_exceed_limit(&self, additional_size: usize) -> bool {
        self.size.load(Ordering::Relaxed) + additional_size > self.size_limit
    }
    
    pub fn iter(&self) -> MemTableIterator {
        MemTableIterator::new(self.skiplist.clone())
    }
}

pub struct MemTableIterator {
    #[allow(dead_code)]
    skiplist: Arc<SkipList>,
    current_key: Option<Key>,
}

impl MemTableIterator {
    fn new(skiplist: Arc<SkipList>) -> Self {
        Self {
            skiplist,
            current_key: None,
        }
    }
    
    pub fn seek(&mut self, key: &[u8]) {
        self.current_key = Some(key.to_vec());
    }
    
    pub fn seek_to_first(&mut self) {
        self.current_key = Some(Vec::new());
    }
    
    pub fn next(&mut self) -> Option<(Key, Value, Timestamp, Operation)> {
        // Note: This is a simplified implementation
        // A full implementation would need to properly iterate through the skip list
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_memtable_basic() {
        let memtable = MemTable::new(1024 * 1024); // 1MB
        
        memtable.put(b"key1".to_vec(), b"value1".to_vec(), 1).unwrap();
        memtable.put(b"key2".to_vec(), b"value2".to_vec(), 2).unwrap();
        memtable.delete(b"key3".to_vec(), 3).unwrap();
        
        let result = memtable.get(b"key1", 10);
        assert!(result.is_some());
        let (value, op) = result.unwrap();
        assert_eq!(value, b"value1");
        assert_eq!(op, Operation::Put);
        
        let result = memtable.get(b"key3", 10);
        assert!(result.is_some());
        let (_, op) = result.unwrap();
        assert_eq!(op, Operation::Delete);
    }
    
    #[test]
    fn test_memtable_size_limit() {
        let memtable = MemTable::new(100); // Very small limit
        
        // This should succeed
        memtable.put(b"key1".to_vec(), b"small".to_vec(), 1).unwrap();
        
        // This should fail due to size limit
        let result = memtable.put(
            b"key_with_very_long_name".to_vec(),
            b"value_with_very_long_content_that_exceeds_limit".to_vec(),
            2
        );
        assert!(result.is_err());
    }
    
    #[test]
    fn test_memtable_scan() {
        let memtable = MemTable::new(1024 * 1024);
        
        memtable.put(b"a".to_vec(), b"value_a".to_vec(), 1).unwrap();
        memtable.put(b"b".to_vec(), b"value_b".to_vec(), 1).unwrap();
        memtable.put(b"c".to_vec(), b"value_c".to_vec(), 1).unwrap();
        memtable.put(b"d".to_vec(), b"value_d".to_vec(), 1).unwrap();
        
        let results = memtable.scan(b"b", b"d", 10);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, b"b");
        assert_eq!(results[1].0, b"c");
    }
}