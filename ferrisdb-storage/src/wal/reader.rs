use super::WALEntry;
use ferrisdb_core::Result;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub struct WALReader {
    reader: BufReader<File>,
}

impl WALReader {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Ok(Self {
            reader: BufReader::new(file),
        })
    }
    
    pub fn read_entry(&mut self) -> Result<Option<WALEntry>> {
        // Read length
        let mut length_buf = [0u8; 4];
        match self.reader.read_exact(&mut length_buf) {
            Ok(_) => {},
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(e.into()),
        }
        
        let length = u32::from_le_bytes(length_buf) as usize;
        
        // Read the rest of the entry
        let mut data = vec![0u8; length + 4];
        data[..4].copy_from_slice(&length_buf);
        self.reader.read_exact(&mut data[4..])?;
        
        Ok(Some(WALEntry::decode(&data)?))
    }
    
    pub fn read_all(&mut self) -> Result<Vec<WALEntry>> {
        let mut entries = Vec::new();
        while let Some(entry) = self.read_entry()? {
            entries.push(entry);
        }
        Ok(entries)
    }
}

impl Iterator for WALReader {
    type Item = Result<WALEntry>;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.read_entry() {
            Ok(Some(entry)) => Some(Ok(entry)),
            Ok(None) => None,
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wal::WALWriter;
    use ferrisdb_core::SyncMode;
    use tempfile::TempDir;
    
    #[test]
    fn test_wal_reader_writer_integration() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        // Write entries
        {
            let writer = WALWriter::new(&wal_path, SyncMode::Full, 1024 * 1024).unwrap();
            
            for i in 0..10 {
                let entry = WALEntry::new_put(
                    format!("key{}", i).into_bytes(),
                    format!("value{}", i).into_bytes(),
                    i as u64,
                );
                writer.append(&entry).unwrap();
            }
            
            writer.sync().unwrap();
        }
        
        // Read entries
        let mut reader = WALReader::new(&wal_path).unwrap();
        let entries = reader.read_all().unwrap();
        
        assert_eq!(entries.len(), 10);
        for (i, entry) in entries.iter().enumerate() {
            assert_eq!(entry.key, format!("key{}", i).into_bytes());
            assert_eq!(entry.value, format!("value{}", i).into_bytes());
            assert_eq!(entry.timestamp, i as u64);
        }
    }
    
    #[test]
    fn test_wal_reader_iterator() {
        let temp_dir = TempDir::new().unwrap();
        let wal_path = temp_dir.path().join("test.wal");
        
        // Write some entries
        {
            let writer = WALWriter::new(&wal_path, SyncMode::Full, 1024 * 1024).unwrap();
            
            for i in 0..5 {
                let entry = if i % 2 == 0 {
                    WALEntry::new_put(
                        format!("key{}", i).into_bytes(),
                        format!("value{}", i).into_bytes(),
                        i as u64,
                    )
                } else {
                    WALEntry::new_delete(
                        format!("key{}", i).into_bytes(),
                        i as u64,
                    )
                };
                writer.append(&entry).unwrap();
            }
        }
        
        // Read using iterator
        let reader = WALReader::new(&wal_path).unwrap();
        let entries: Result<Vec<_>> = reader.collect();
        let entries = entries.unwrap();
        
        assert_eq!(entries.len(), 5);
        assert_eq!(entries[1].operation, ferrisdb_core::Operation::Delete);
        assert_eq!(entries[1].value, Vec::<u8>::new());
    }
}