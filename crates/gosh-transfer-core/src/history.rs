// SPDX-License-Identifier: AGPL-3.0
// Gosh Transfer Core - Transfer history persistence
//
// Stores completed transfer records in a local JSON file.

use crate::types::AppError;
use gosh_lan_transfer::TransferRecord;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

/// Maximum number of history entries to keep
const MAX_HISTORY_ENTRIES: usize = 100;

/// File-based transfer history storage
pub struct TransferHistory {
    records: RwLock<Vec<TransferRecord>>,
    file_path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HistoryFile {
    records: Vec<TransferRecord>,
}

impl TransferHistory {
    /// Create a new history store, loading from disk if available
    pub fn new() -> Result<Self, AppError> {
        let file_path = Self::get_history_path()?;

        let records = if file_path.exists() {
            let content = fs::read_to_string(&file_path)
                .map_err(|e| AppError::FileIo(format!("Failed to read history: {}", e)))?;

            let file: HistoryFile = serde_json::from_str(&content).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse history, starting fresh: {}", e);
                HistoryFile {
                    records: Vec::new(),
                }
            });

            file.records
        } else {
            Vec::new()
        };

        Ok(Self {
            records: RwLock::new(records),
            file_path,
        })
    }

    /// Get the path to the history file
    fn get_history_path() -> Result<PathBuf, AppError> {
        let config_dir = directories::ProjectDirs::from("com", "gosh", "transfer")
            .ok_or_else(|| AppError::FileIo("Could not determine config directory".to_string()))?
            .config_dir()
            .to_path_buf();

        // Ensure the directory exists
        fs::create_dir_all(&config_dir)
            .map_err(|e| AppError::FileIo(format!("Failed to create config dir: {}", e)))?;

        Ok(config_dir.join("history.json"))
    }

    /// Persist history to disk
    fn persist(&self) -> Result<(), AppError> {
        let records = self.records.read().unwrap();
        let file = HistoryFile {
            records: records.clone(),
        };

        let content = serde_json::to_string_pretty(&file)
            .map_err(|e| AppError::Serialization(format!("Failed to serialize history: {}", e)))?;

        fs::write(&self.file_path, content)
            .map_err(|e| AppError::FileIo(format!("Failed to write history: {}", e)))?;

        Ok(())
    }

    /// Get all transfer records
    pub fn list(&self) -> Vec<TransferRecord> {
        self.records.read().unwrap().clone()
    }

    /// Add a new transfer record
    pub fn add(&self, record: TransferRecord) -> Result<(), AppError> {
        {
            let mut records = self.records.write().unwrap();

            // Add new record at the beginning (most recent first)
            records.insert(0, record);

            // Trim to max entries
            if records.len() > MAX_HISTORY_ENTRIES {
                records.truncate(MAX_HISTORY_ENTRIES);
            }
        }

        self.persist()
    }

    /// Clear all history
    pub fn clear(&self) -> Result<(), AppError> {
        {
            let mut records = self.records.write().unwrap();
            records.clear();
        }

        self.persist()
    }

    /// Get the count of history entries
    pub fn count(&self) -> usize {
        self.records.read().unwrap().len()
    }
}

impl Default for TransferHistory {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            records: RwLock::new(Vec::new()),
            file_path: PathBuf::from("history.json"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_history_constant() {
        assert_eq!(MAX_HISTORY_ENTRIES, 100);
    }
}
