use ferrisdb_core::{Key, Value, Operation, Timestamp};
use std::cmp::Ordering;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::ops::Deref;
use crossbeam::epoch::{self, Atomic, Guard, Owned, Shared};
use parking_lot::Mutex;
use rand::Rng;

const MAX_HEIGHT: usize = 12;
const BRANCHING_FACTOR: u32 = 4;

#[derive(Debug, Clone)]
pub struct InternalKey {
    pub user_key: Key,
    pub timestamp: Timestamp,
    pub operation: Operation,
}

impl InternalKey {
    fn new(user_key: Key, timestamp: Timestamp, operation: Operation) -> Self {
        Self { user_key, timestamp, operation }
    }
    
    fn compare(&self, other: &Self) -> Ordering {
        match self.user_key.cmp(&other.user_key) {
            Ordering::Equal => {
                // Newer timestamps come first (descending order)
                match other.timestamp.cmp(&self.timestamp) {
                    Ordering::Equal => Ordering::Equal,
                    other => other,
                }
            }
            other => other,
        }
    }
}

struct Node {
    key: InternalKey,
    value: Value,
    next: Vec<Atomic<Node>>,
}

impl Node {
    fn new(key: InternalKey, value: Value, height: usize) -> Self {
        let mut next = Vec::with_capacity(height);
        for _ in 0..height {
            next.push(Atomic::null());
        }
        
        Self { key, value, next }
    }
    
    fn head(height: usize) -> Self {
        Self::new(
            InternalKey::new(Vec::new(), 0, Operation::Put),
            Vec::new(),
            height,
        )
    }
}

pub struct SkipList {
    head: Atomic<Node>,
    height: AtomicUsize,
    size: AtomicUsize,
    rng: Mutex<rand::rngs::ThreadRng>,
}

impl SkipList {
    pub fn new() -> Self {
        let head = Node::head(MAX_HEIGHT);
        
        Self {
            head: Atomic::new(head),
            height: AtomicUsize::new(1),
            size: AtomicUsize::new(0),
            rng: Mutex::new(rand::thread_rng()),
        }
    }
    
    fn random_height(&self) -> usize {
        let mut height = 1;
        let mut rng = self.rng.lock();
        
        while height < MAX_HEIGHT && rng.gen_ratio(1, BRANCHING_FACTOR) {
            height += 1;
        }
        
        height
    }
    
    pub fn insert(&self, user_key: Key, value: Value, timestamp: Timestamp, operation: Operation) {
        let guard = &epoch::pin();
        let key = InternalKey::new(user_key, timestamp, operation);
        let height = self.random_height();
        
        // Update max height if necessary
        loop {
            let current_height = self.height.load(AtomicOrdering::Acquire);
            if height <= current_height {
                break;
            }
            
            if self.height.compare_exchange(
                current_height,
                height,
                AtomicOrdering::Release,
                AtomicOrdering::Acquire,
            ).is_ok() {
                break;
            }
        }
        
        let mut preds: Vec<Shared<Node>> = vec![Shared::null(); height];
        let mut succs: Vec<Shared<Node>> = vec![Shared::null(); height];
        
        loop {
            if self.find(&key, &mut preds, &mut succs, guard) {
                // Key already exists, we don't update in skip list
                // (newer version should be inserted as separate entry)
                break;
            }
            
            let new_node = Owned::new(Node::new(key.clone(), value.clone(), height));
            
            // Set next pointers of new node
            for i in 0..height {
                new_node.deref().next[i].store(succs[i], AtomicOrdering::Relaxed);
            }
            
            let new_node_shared = new_node.into_shared(guard);
            
            // Try to link the new node
            match unsafe { preds[0].as_ref() }.unwrap().next[0].compare_exchange(
                succs[0],
                new_node_shared,
                AtomicOrdering::Release,
                AtomicOrdering::Acquire,
                guard,
            ) {
                Ok(_) => {
                    // Successfully linked at bottom level, link other levels
                    for i in 1..height {
                        loop {
                            match unsafe { preds[i].as_ref() }.unwrap().next[i].compare_exchange(
                                succs[i],
                                new_node_shared,
                                AtomicOrdering::Release,
                                AtomicOrdering::Acquire,
                                guard,
                            ) {
                                Ok(_) => break,
                                Err(_) => {
                                    // Re-find predecessors for this level
                                    self.find(&key, &mut preds, &mut succs, guard);
                                }
                            }
                        }
                    }
                    
                    self.size.fetch_add(1, AtomicOrdering::Relaxed);
                    break;
                }
                Err(_) => {
                    // CAS failed, retry
                    continue;
                }
            }
        }
    }
    
    fn find<'g>(
        &self,
        key: &InternalKey,
        preds: &mut [Shared<'g, Node>],
        succs: &mut [Shared<'g, Node>],
        guard: &'g Guard,
    ) -> bool {
        let mut pred = self.head.load(AtomicOrdering::Acquire, guard);
        
        for level in (0..self.height.load(AtomicOrdering::Acquire)).rev() {
            let mut curr = unsafe { pred.as_ref() }.unwrap().next[level].load(AtomicOrdering::Acquire, guard);
            
            while !curr.is_null() {
                let curr_ref = unsafe { curr.as_ref() }.unwrap();
                
                match key.compare(&curr_ref.key) {
                    Ordering::Greater => {
                        pred = curr;
                        curr = curr_ref.next[level].load(AtomicOrdering::Acquire, guard);
                    }
                    _ => break,
                }
            }
            
            if level < preds.len() {
                preds[level] = pred;
                succs[level] = curr;
            }
        }
        
        !succs[0].is_null() && unsafe { succs[0].as_ref() }.unwrap().key.compare(key) == Ordering::Equal
    }
    
    pub fn get(&self, user_key: &[u8], timestamp: Timestamp) -> Option<(Value, Operation)> {
        let guard = &epoch::pin();
        
        // First, find the position where this key would be
        let search_key = InternalKey::new(user_key.to_vec(), u64::MAX, Operation::Put);
        let mut preds = vec![Shared::null(); 1];
        let mut succs = vec![Shared::null(); 1];
        
        self.find(&search_key, &mut preds, &mut succs, guard);
        
        // Now scan from this position to find the right version
        let mut curr = succs[0];
        
        while !curr.is_null() {
            let curr_ref = unsafe { curr.as_ref() }.unwrap();
            
            if curr_ref.key.user_key != user_key {
                break;
            }
            
            if curr_ref.key.timestamp <= timestamp {
                return Some((curr_ref.value.clone(), curr_ref.key.operation));
            }
            
            curr = curr_ref.next[0].load(AtomicOrdering::Acquire, guard);
        }
        
        None
    }
    
    pub fn scan(&self, start_key: &[u8], end_key: &[u8], timestamp: Timestamp) -> Vec<(Key, Value)> {
        let guard = &epoch::pin();
        let mut result = Vec::new();
        let mut seen_keys = std::collections::HashSet::new();
        
        let search_key = InternalKey::new(start_key.to_vec(), timestamp, Operation::Put);
        let mut preds = vec![Shared::null(); 1];
        let mut succs = vec![Shared::null(); 1];
        
        self.find(&search_key, &mut preds, &mut succs, guard);
        
        let mut curr = succs[0];
        
        while !curr.is_null() {
            let curr_ref = unsafe { curr.as_ref() }.unwrap();
            
            if curr_ref.key.user_key.as_slice() >= end_key {
                break;
            }
            
            if curr_ref.key.timestamp <= timestamp && !seen_keys.contains(&curr_ref.key.user_key) {
                if curr_ref.key.operation == Operation::Put {
                    result.push((curr_ref.key.user_key.clone(), curr_ref.value.clone()));
                }
                seen_keys.insert(curr_ref.key.user_key.clone());
            }
            
            curr = curr_ref.next[0].load(AtomicOrdering::Acquire, guard);
        }
        
        result
    }
    
    pub fn size(&self) -> usize {
        self.size.load(AtomicOrdering::Relaxed)
    }
}

impl Drop for SkipList {
    fn drop(&mut self) {
        let guard = &epoch::pin();
        
        let mut curr = self.head.load(AtomicOrdering::Acquire, guard);
        while !curr.is_null() {
            let next = unsafe { curr.as_ref() }.unwrap().next[0].load(AtomicOrdering::Acquire, guard);
            unsafe {
                guard.defer_destroy(curr);
            }
            curr = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_skiplist_basic() {
        let sl = SkipList::new();
        
        sl.insert(b"key1".to_vec(), b"value1".to_vec(), 1, Operation::Put);
        sl.insert(b"key2".to_vec(), b"value2".to_vec(), 2, Operation::Put);
        sl.insert(b"key3".to_vec(), b"value3".to_vec(), 3, Operation::Put);
        
        assert_eq!(sl.size(), 3);
        
        let result = sl.get(b"key2", 5);
        assert!(result.is_some());
        let (value, op) = result.unwrap();
        assert_eq!(value, b"value2");
        assert_eq!(op, Operation::Put);
    }
    
    #[test]
    fn test_skiplist_versions() {
        let sl = SkipList::new();
        
        // Insert multiple versions of the same key
        sl.insert(b"key1".to_vec(), b"value1".to_vec(), 1, Operation::Put);
        sl.insert(b"key1".to_vec(), b"value2".to_vec(), 3, Operation::Put);
        sl.insert(b"key1".to_vec(), b"value3".to_vec(), 5, Operation::Put);
        
        // Read at different timestamps
        let result = sl.get(b"key1", 2);
        assert_eq!(result.unwrap().0, b"value1");
        
        let result = sl.get(b"key1", 4);
        assert_eq!(result.unwrap().0, b"value2");
        
        let result = sl.get(b"key1", 6);
        assert_eq!(result.unwrap().0, b"value3");
    }
    
    #[test]
    fn test_skiplist_delete() {
        let sl = SkipList::new();
        
        sl.insert(b"key1".to_vec(), b"value1".to_vec(), 1, Operation::Put);
        sl.insert(b"key1".to_vec(), Vec::new(), 3, Operation::Delete);
        
        // Before delete
        let result = sl.get(b"key1", 2);
        assert_eq!(result.unwrap().1, Operation::Put);
        
        // After delete
        let result = sl.get(b"key1", 4);
        assert_eq!(result.unwrap().1, Operation::Delete);
    }
}