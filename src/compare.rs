use std::collections::HashMap;

use crate::photo::PhotoInfo;
use crate::photos::PhotoAsset;

/// Result of comparing source photos with library photos
#[derive(Debug)]
pub struct ComparisonResult {
    /// Photos that exist in source but not in library (need to be added)
    pub missing_in_library: Vec<PhotoInfo>,

    /// Photos that exist in both (by hash)
    pub already_in_library: Vec<PhotoInfo>,

    /// Duplicate photos in source (same hash, different paths)
    pub duplicates_in_source: Vec<Vec<PhotoInfo>>,
}

impl ComparisonResult {
    /// Get the total number of photos scanned
    pub fn total_scanned(&self) -> usize {
        self.missing_in_library.len() + self.already_in_library.len()
    }

    /// Get the number of photos to be added
    pub fn to_add(&self) -> usize {
        self.missing_in_library.len()
    }

    /// Get the number of photos already present
    pub fn already_present(&self) -> usize {
        self.already_in_library.len()
    }

    /// Get the number of duplicate groups
    pub fn duplicate_groups(&self) -> usize {
        self.duplicates_in_source.len()
    }
}

/// Compare source photos with library photos
///
/// Returns a ComparisonResult indicating which photos need to be added
pub fn compare_photos(
    source_photos: Vec<PhotoInfo>,
    library_hash_map: &HashMap<String, PhotoAsset>,
) -> ComparisonResult {
    let mut missing_in_library = Vec::new();
    let mut already_in_library = Vec::new();
    let mut hash_to_photos: HashMap<String, Vec<PhotoInfo>> = HashMap::new();

    // Group source photos by hash
    for photo in source_photos {
        hash_to_photos
            .entry(photo.hash.clone())
            .or_insert_with(Vec::new)
            .push(photo);
    }

    // Identify duplicates and missing photos
    let mut duplicates_in_source = Vec::new();

    for (hash, photos) in hash_to_photos {
        if library_hash_map.contains_key(&hash) {
            // Photo(s) with this hash already exist in library
            already_in_library.extend(photos);
        } else {
            // Photo(s) with this hash are missing from library
            if photos.len() > 1 {
                // Multiple files with same hash in source
                duplicates_in_source.push(photos.clone());
            }
            // Only add one copy if there are duplicates
            missing_in_library.push(photos.into_iter().next().unwrap());
        }
    }

    ComparisonResult {
        missing_in_library,
        already_in_library,
        duplicates_in_source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_compare_photos_all_missing() {
        let source_photos = vec![
            PhotoInfo {
                path: PathBuf::from("photo1.jpg"),
                hash: "hash1".to_string(),
                size: 1000,
            },
            PhotoInfo {
                path: PathBuf::from("photo2.jpg"),
                hash: "hash2".to_string(),
                size: 2000,
            },
        ];

        let library_hash_map = HashMap::new();

        let result = compare_photos(source_photos, &library_hash_map);

        assert_eq!(result.to_add(), 2);
        assert_eq!(result.already_present(), 0);
        assert_eq!(result.duplicate_groups(), 0);
    }

    #[test]
    fn test_compare_photos_some_present() {
        let source_photos = vec![
            PhotoInfo {
                path: PathBuf::from("photo1.jpg"),
                hash: "hash1".to_string(),
                size: 1000,
            },
            PhotoInfo {
                path: PathBuf::from("photo2.jpg"),
                hash: "hash2".to_string(),
                size: 2000,
            },
        ];

        let mut library_hash_map = HashMap::new();
        library_hash_map.insert(
            "hash1".to_string(),
            PhotoAsset {
                uuid: "uuid1".to_string(),
                filename: "photo1.jpg".to_string(),
                directory: "dir1".to_string(),
                file_size: Some(1000),
            },
        );

        let result = compare_photos(source_photos, &library_hash_map);

        assert_eq!(result.to_add(), 1);
        assert_eq!(result.already_present(), 1);
    }

    #[test]
    fn test_compare_photos_with_duplicates() {
        let source_photos = vec![
            PhotoInfo {
                path: PathBuf::from("photo1.jpg"),
                hash: "hash1".to_string(),
                size: 1000,
            },
            PhotoInfo {
                path: PathBuf::from("photo1_copy.jpg"),
                hash: "hash1".to_string(),
                size: 1000,
            },
        ];

        let library_hash_map = HashMap::new();

        let result = compare_photos(source_photos, &library_hash_map);

        assert_eq!(result.to_add(), 1); // Only one copy should be added
        assert_eq!(result.duplicate_groups(), 1);
    }
}
