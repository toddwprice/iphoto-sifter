use thiserror::Error;

#[derive(Error, Debug)]
pub enum SifterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),

    #[error("Invalid Photos library: {0}")]
    InvalidLibrary(String),

    #[error("Photo processing error: {0}")]
    PhotoProcessing(String),

    #[error("Hash calculation error: {0}")]
    HashError(String),

    #[error("Library write error: {0}")]
    LibraryWrite(String),
}

pub type Result<T> = std::result::Result<T, SifterError>;
