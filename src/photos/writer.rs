use chrono::Utc;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::error::{Result, SifterError};
use crate::photo::PhotoInfo;

/// Result of adding a photo to the library
#[derive(Debug)]
pub struct AddResult {
    pub source_path: PathBuf,
    pub library_path: PathBuf,
    pub uuid: String,
    pub success: bool,
    pub error: Option<String>,
}

/// Add a photo to the Photos library
///
/// IMPORTANT: This function modifies the Photos.sqlite database.
/// The library should NOT be open in Photos.app when this runs.
pub fn add_photo_to_library<P: AsRef<Path>>(
    library_path: P,
    photo: &PhotoInfo,
    dry_run: bool,
) -> Result<AddResult> {
    let library_path = library_path.as_ref();
    let originals_dir = library_path.join("originals");

    // Generate a UUID for the new asset
    let asset_uuid = Uuid::new_v4().to_string().to_uppercase();

    // Determine the directory structure for the photo
    // Photos.app uses a UUID-based directory structure
    // Format: originals/{first char}/{UUID}/
    let first_char = asset_uuid.chars().next().unwrap();
    let photo_dir = format!("{}/{}", first_char, asset_uuid);
    let target_dir = originals_dir.join(&photo_dir);

    // Get the filename
    let filename = photo
        .path
        .file_name()
        .ok_or_else(|| SifterError::PhotoProcessing("Invalid filename".to_string()))?
        .to_string_lossy()
        .to_string();

    let target_path = target_dir.join(&filename);

    if dry_run {
        return Ok(AddResult {
            source_path: photo.path.clone(),
            library_path: target_path,
            uuid: asset_uuid,
            success: true,
            error: None,
        });
    }

    // Create the directory structure
    fs::create_dir_all(&target_dir).map_err(|e| {
        SifterError::LibraryWrite(format!(
            "Failed to create directory {}: {}",
            target_dir.display(),
            e
        ))
    })?;

    // Copy the file
    fs::copy(&photo.path, &target_path).map_err(|e| {
        SifterError::LibraryWrite(format!(
            "Failed to copy file to library: {}",
            e
        ))
    })?;

    // Add entry to database
    let db_path = library_path.join("database/Photos.sqlite");
    let conn = Connection::open(&db_path)?;

    // This is a simplified version. The actual Photos.app database schema is complex
    // and includes many tables and relationships. This is a basic implementation.
    //
    // WARNING: Directly modifying Photos.app database can corrupt the library!
    // This should be considered experimental.

    let now = Utc::now().timestamp();

    // Insert into ZASSET table
    // Note: This is a simplified schema. Real Photos.app has many more columns.
    conn.execute(
        "INSERT INTO ZASSET (
            ZUUID,
            ZFILENAME,
            ZDIRECTORY,
            ZFILESIZE,
            ZDATECREATED,
            ZADDEDDATE,
            ZTRASHEDSTATE,
            ZKIND,
            ZVISIBILITYSTATE
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            &asset_uuid,
            &filename,
            &photo_dir,
            photo.size as i64,
            now,
            now,
            0, // Not trashed
            0, // Photo (not video)
            0, // Visible
        ],
    )
    .map_err(|e| {
        // If database insert fails, clean up the copied file
        let _ = fs::remove_file(&target_path);
        let _ = fs::remove_dir(&target_dir);
        SifterError::Database(e)
    })?;

    Ok(AddResult {
        source_path: photo.path.clone(),
        library_path: target_path,
        uuid: asset_uuid,
        success: true,
        error: None,
    })
}

/// Create a backup of the Photos.sqlite database
pub fn backup_database<P: AsRef<Path>>(library_path: P) -> Result<PathBuf> {
    let db_path = library_path.as_ref().join("database/Photos.sqlite");
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    let backup_path = library_path
        .as_ref()
        .join(format!("database/Photos.sqlite.backup_{}", timestamp));

    fs::copy(&db_path, &backup_path).map_err(|e| {
        SifterError::LibraryWrite(format!("Failed to create backup: {}", e))
    })?;

    Ok(backup_path)
}
