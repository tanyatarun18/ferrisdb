// Storage engine implementation
use crate::StorageConfig;

pub struct StorageEngine {
    #[allow(dead_code)]
    config: StorageConfig,
}

impl StorageEngine {
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }
}