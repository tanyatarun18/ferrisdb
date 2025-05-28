use ferrisdb_core::{CompressionType, SyncMode};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct StorageConfig {
    // Directories
    pub data_dir: PathBuf,
    pub wal_dir: PathBuf,
    
    // WAL settings
    pub wal_sync_mode: SyncMode,
    pub wal_size_limit: usize,
    
    // MemTable settings
    pub memtable_size: usize,
    pub max_immutable_memtables: usize,
    
    // SSTable settings
    pub block_size: usize,
    pub compression: CompressionType,
    
    // Compaction settings
    pub level0_file_num_compaction_trigger: i32,
    pub max_bytes_for_level_base: u64,
    pub max_bytes_for_level_multiplier: f64,
    
    // Cache settings
    pub block_cache_size: usize,
    pub bloom_filter_bits_per_key: i32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            wal_dir: PathBuf::from("./data/wal"),
            wal_sync_mode: SyncMode::Normal,
            wal_size_limit: 64 * 1024 * 1024, // 64MB
            memtable_size: 4 * 1024 * 1024, // 4MB
            max_immutable_memtables: 2,
            block_size: 4 * 1024, // 4KB
            compression: CompressionType::Lz4,
            level0_file_num_compaction_trigger: 4,
            max_bytes_for_level_base: 10 * 1024 * 1024, // 10MB
            max_bytes_for_level_multiplier: 10.0,
            block_cache_size: 128 * 1024 * 1024, // 128MB
            bloom_filter_bits_per_key: 10,
        }
    }
}