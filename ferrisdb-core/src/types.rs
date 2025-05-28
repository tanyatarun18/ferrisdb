use serde::{Deserialize, Serialize};

pub type Key = Vec<u8>;
pub type Value = Vec<u8>;
pub type SequenceNumber = u64;
pub type Timestamp = u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
    Put,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyValue {
    pub key: Key,
    pub value: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimestampedKeyValue {
    pub key: Key,
    pub value: Value,
    pub timestamp: Timestamp,
    pub operation: Operation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Lz4,
    Snappy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncMode {
    None,
    Normal,
    Full,
}