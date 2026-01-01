use std::path::{Path, PathBuf};

use crate::error::{Result, SifterError};

/// Represents a Photos.app library
#[derive(Debug)]
pub struct PhotosLibrary {
    pub path: PathBuf,
}

impl PhotosLibrary {
    /// Open a Photos.app library at the given path
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        // Validate that this is a Photos library
        if !path.exists() {
            return Err(SifterError::InvalidLibrary(format!(
                "Library path does not exist: {}",
                path.display()
            )));
        }

        if !path.is_dir() {
            return Err(SifterError::InvalidLibrary(format!(
                "Library path is not a directory: {}",
                path.display()
            )));
        }

        // Check for Photos.sqlite database
        let db_path = path.join("database/Photos.sqlite");
        if !db_path.exists() {
            return Err(SifterError::InvalidLibrary(format!(
                "Not a valid Photos library (missing database/Photos.sqlite): {}",
                path.display()
            )));
        }

        // Check for originals directory
        let originals_path = path.join("originals");
        if !originals_path.exists() {
            return Err(SifterError::InvalidLibrary(format!(
                "Not a valid Photos library (missing originals directory): {}",
                path.display()
            )));
        }

        Ok(Self { path })
    }

    /// Get the path to the database directory
    pub fn database_path(&self) -> PathBuf {
        self.path.join("database/Photos.sqlite")
    }

    /// Get the path to the originals directory
    pub fn originals_path(&self) -> PathBuf {
        self.path.join("originals")
    }

    /// Get the path to the resources directory
    pub fn resources_path(&self) -> PathBuf {
        self.path.join("resources")
    }

    /// Check if the library appears to be in use (has .lock files or WAL files)
    pub fn is_in_use(&self) -> bool {
        // Check for SQLite WAL (Write-Ahead Log) files which indicate active use
        let wal_path = self.path.join("database/Photos.sqlite-wal");
        let shm_path = self.path.join("database/Photos.sqlite-shm");

        wal_path.exists() || shm_path.exists()
    }
}
