use std::string::{String, ToString};

use rusqlite::Error as RusqliteError;
use rusqlite_migration::Error as MigrationError;
use thiserror::Error;

// ERRORS
// ================================================================================================

/// Errors generated from the `SQLite` store.
#[derive(Debug, Error)]
pub enum SqliteStoreError {
    #[error("Database error: {0}")]
    DatabaseError(String),
    #[error("Migration error: {0}")]
    MigrationError(String),
    #[error("No settings table in the database")]
    MissingSettingsTable,
    #[error("Migration hashes mismatch")]
    MigrationHashMismatch,
    #[error("Failed to decode hex string: {0}")]
    HexDecodeError(String),
}

impl From<RusqliteError> for SqliteStoreError {
    fn from(err: RusqliteError) -> Self {
        SqliteStoreError::DatabaseError(err.to_string())
    }
}

impl From<MigrationError> for SqliteStoreError {
    fn from(err: MigrationError) -> Self {
        SqliteStoreError::MigrationError(err.to_string())
    }
}
