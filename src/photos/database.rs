use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Result, SifterError};

/// Represents a photo asset in the Photos library
#[derive(Debug, Clone)]
pub struct PhotoAsset {
    pub uuid: String,
    pub filename: String,
    pub directory: String,
    pub file_size: Option<i64>,
}

/// Open a Photos.app database in read-only mode
pub fn open_database<P: AsRef<Path>>(library_path: P) -> Result<Connection> {
    let db_path = library_path.as_ref().join("database/Photos.sqlite");

    if !db_path.exists() {
        return Err(SifterError::InvalidLibrary(format!(
            "Photos.sqlite not found at {}",
            db_path.display()
        )));
    }

    Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| SifterError::Database(e))
}

/// Read all photo assets from the Photos library
pub fn read_all_assets(conn: &Connection) -> Result<Vec<PhotoAsset>> {
    // Photos.app database schema varies by version, but we'll query the main asset table
    // The ZASSET table contains the primary asset information
    let mut stmt = conn
        .prepare(
            "SELECT ZUUID, ZFILENAME, ZDIRECTORY, ZFILESIZE
             FROM ZASSET
             WHERE ZTRASHEDSTATE = 0 AND ZKIND = 0",
        )
        .map_err(|e| {
            SifterError::Database(e)
        })?;

    let assets = stmt
        .query_map([], |row| {
            Ok(PhotoAsset {
                uuid: row.get(0)?,
                filename: row.get(1)?,
                directory: row.get(2)?,
                file_size: row.get(3)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    Ok(assets)
}

/// Build a map of file hashes to assets
/// This requires reading the actual files from the originals directory
pub fn build_hash_map<P: AsRef<Path>>(
    library_path: P,
    assets: &[PhotoAsset],
) -> Result<HashMap<String, PhotoAsset>> {
    let originals_path = library_path.as_ref().join("originals");
    let mut hash_map = HashMap::new();

    for asset in assets {
        // Construct the file path: originals/{directory}/{filename}
        let file_path = originals_path.join(&asset.directory).join(&asset.filename);

        if !file_path.exists() {
            // Skip assets whose files are missing
            continue;
        }

        // Calculate hash
        match crate::photo::hasher::hash_file(&file_path) {
            Ok(hash) => {
                hash_map.insert(hash, asset.clone());
            }
            Err(e) => {
                eprintln!(
                    "Warning: Failed to hash {}: {}",
                    file_path.display(),
                    e
                );
            }
        }
    }

    Ok(hash_map)
}

/// Get the originals directory path
pub fn get_originals_path<P: AsRef<Path>>(library_path: P) -> PathBuf {
    library_path.as_ref().join("originals")
}
