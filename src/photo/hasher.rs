use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use crate::error::{Result, SifterError};

/// Calculate SHA-256 hash of a file
pub fn hash_file<P: AsRef<Path>>(path: P) -> Result<String> {
    let file = File::open(path.as_ref()).map_err(|e| {
        SifterError::HashError(format!(
            "Failed to open file {}: {}",
            path.as_ref().display(),
            e
        ))
    })?;

    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192]; // 8KB buffer

    loop {
        let bytes_read = reader.read(&mut buffer).map_err(|e| {
            SifterError::HashError(format!(
                "Failed to read file {}: {}",
                path.as_ref().display(),
                e
            ))
        })?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_hash_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(b"Hello, World!").unwrap();
        temp_file.flush().unwrap();

        let hash = hash_file(temp_file.path()).unwrap();

        // Expected SHA-256 hash of "Hello, World!"
        assert_eq!(
            hash,
            "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
        );
    }
}
