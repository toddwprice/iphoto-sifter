use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::error::Result;
use crate::photo::hasher;

/// Supported photo file extensions
const PHOTO_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "heic", "heif", // Common formats
    "cr2", "nef", "arw", "dng", "orf", "rw2", "pef", "srw", // RAW formats
    "tiff", "tif", "gif", "bmp", // Additional formats
    "mov", "mp4", "m4v", // Videos (Photos.app supports these)
];

/// Information about a scanned photo
#[derive(Debug, Clone)]
pub struct PhotoInfo {
    pub path: PathBuf,
    pub hash: String,
    pub size: u64,
}

/// Check if a file has a supported photo extension
pub fn is_photo_file<P: AsRef<Path>>(path: P) -> bool {
    if let Some(ext) = path.as_ref().extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        PHOTO_EXTENSIONS.contains(&ext_str.as_str())
    } else {
        false
    }
}

/// Scan a directory for photos and calculate their hashes
pub fn scan_directory<P: AsRef<Path>>(
    path: P,
    recursive: bool,
) -> Result<Vec<PhotoInfo>> {
    let mut photos = Vec::new();
    let walker = if recursive {
        WalkDir::new(path).follow_links(true)
    } else {
        WalkDir::new(path).max_depth(1)
    };

    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if !is_photo_file(path) {
            continue;
        }

        // Calculate hash
        let hash = hasher::hash_file(path)?;

        // Get file size
        let size = entry.metadata()?.len();

        photos.push(PhotoInfo {
            path: path.to_path_buf(),
            hash,
            size,
        });
    }

    Ok(photos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_photo_file() {
        assert!(is_photo_file("test.jpg"));
        assert!(is_photo_file("test.JPEG"));
        assert!(is_photo_file("test.png"));
        assert!(is_photo_file("test.heic"));
        assert!(is_photo_file("test.cr2"));
        assert!(is_photo_file("test.NEF"));

        assert!(!is_photo_file("test.txt"));
        assert!(!is_photo_file("test.pdf"));
        assert!(!is_photo_file("test"));
    }
}
