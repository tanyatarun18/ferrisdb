use super::WALEntry;
use ferrisdb_core::{Result, Error, SyncMode};
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;

pub struct WALWriter {
    file: Arc<Mutex<BufWriter<File>>>,
    path: PathBuf,
    size: AtomicU64,
    sync_mode: SyncMode,
    size_limit: u64,
}

impl WALWriter {
    pub fn new(path: impl AsRef<Path>, sync_mode: SyncMode, size_limit: u64) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        std::fs::create_dir_all(path.parent().unwrap())?;
        
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
            
        let metadata = file.metadata()?;
        let size = metadata.len();
        
        Ok(Self {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
            path,
            size: AtomicU64::new(size),
            sync_mode,
            size_limit,
        })
    }
    
    pub fn append(&self, entry: &WALEntry) -> Result<()> {
        let encoded = entry.encode();
        let entry_size = encoded.len() as u64;
        
        // Check if we need to rotate
        if self.size.load(Ordering::Relaxed) + entry_size > self.size_limit {
            return Err(Error::StorageEngine("WAL file size limit reached".to_string()));
        }
        
        let mut file = self.file.lock();
        file.write_all(&encoded)?;
        
        match self.sync_mode {
            SyncMode::None => {},
            SyncMode::Normal => {
                file.flush()?;
            },
            SyncMode::Full => {
                file.flush()?;
                file.get_ref().sync_all()?;
            },
        }
        
        self.size.fetch_add(entry_size, Ordering::Relaxed);
        Ok(())
    }
    
    pub fn sync(&self) -> Result<()> {
        let mut file = self.file.lock();
        file.flush()?;
        file.get_ref().sync_all()?;
        Ok(())
    }
    
    pub fn size(&self) -> u64 {
        self.size.load(Ordering::Relaxed)
    }
    
    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_wal_writer_basic() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        let writer = WALWriter::new(&wal_path, SyncMode::Normal, 1024 * 1024).unwrap();
        
        let entry = WALEntry::new_put(
            b"key1".to_vec(),
            b"value1".to_vec(),
            1,
        );
        
        writer.append(&entry).unwrap();
        writer.sync().unwrap();
        
        assert!(writer.size() > 0);
        assert!(wal_path.exists());
    }
    
    #[test]
    fn test_wal_size_limit() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        // Very small size limit
        let writer = WALWriter::new(&wal_path, SyncMode::None, 50).unwrap();
        
        let entry = WALEntry::new_put(
            b"key_with_long_name".to_vec(),
            b"value_with_long_content".to_vec(),
            1,
        );
        
        // This should exceed the size limit
        let result = writer.append(&entry);
        assert!(result.is_err());
    }
}