//! Write-Ahead Log (WAL) implementation
//! 
//! The WAL provides durability by persisting all write operations to disk
//! before they are applied to the in-memory data structures. Each entry
//! in the WAL contains:
//! 
//! - Length and checksum for corruption detection
//! - Timestamp for ordering
//! - Operation type (Put or Delete)
//! - Key and value data
//! 
//! # Example
//! 
//! ```no_run
//! use ferrisdb_storage::wal::{WALWriter, WALReader, WALEntry};
//! use ferrisdb_core::SyncMode;
//! 
//! // Write to WAL
//! let writer = WALWriter::new("path/to/wal", SyncMode::Normal, 64 * 1024 * 1024)?;
//! let entry = WALEntry::new_put(b"key".to_vec(), b"value".to_vec(), 1);
//! writer.append(&entry)?;
//! 
//! // Read from WAL
//! let mut reader = WALReader::new("path/to/wal")?;
//! let entries = reader.read_all()?;
//! # Ok::<(), ferrisdb_core::Error>(())
//! ```

mod writer;
mod reader;
mod log_entry;

pub use writer::WALWriter;
pub use reader::WALReader;
pub use log_entry::WALEntry;