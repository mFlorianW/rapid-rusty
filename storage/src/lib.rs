//! Storage Modul for the laptimer
//!
//! Provides the interfaces and implementation to store and load session and track data on linux based systems.

use async_trait::async_trait;
use common::session::Session;
use std::{fs::exists, io};
use tokio::fs::read_dir;

/// An asynchronous trait for storing and retrieving [`Session`] data.
///
/// This trait defines the interface for saving, loading, deleting,
/// and listing sessions in an asynchronous context, such as file I/O
/// typically backed by [`tokio`] runtime.
///
/// Implementors must ensure non-blocking operations using async APIs.
///
/// # Errors
///
/// All methods return [`std::io::Error`] if I/O fails (e.g. file missing, permission error).
#[async_trait]
pub trait SessionStorage: Send + Sync {
    /// Saves a [`Session`] asynchronously.
    ///
    /// Overwrites any existing session with the same ID.
    async fn save(&self, session: &Session) -> io::Result<()>;

    /// Loads a [`Session`] with the given ID.
    ///
    /// Returns an error if the session does not exist or reading fails.
    async fn load(&self, id: &str) -> io::Result<Session>;

    /// Deletes the session with the given ID.
    ///
    /// Returns an error if deletion fails or the session is not found.
    async fn delete(&self, id: &str) -> io::Result<()>;

    /// Lists of all session ids currently stored.
    ///
    /// Returns a list of all sessions, or an error on failure.
    async fn ids(&self) -> io::Result<Vec<String>>;
}

pub struct SessionFsStorage {
    root_dir: String,
}

impl SessionFsStorage {
    pub fn new(root_dir: &str) -> Self {
        SessionFsStorage {
            root_dir: root_dir.to_string(),
        }
    }
}

#[async_trait]
impl SessionStorage for SessionFsStorage {
    async fn save(&self, session: &Session) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    async fn load(&self, id: &str) -> io::Result<Session> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    async fn delete(&self, id: &str) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::Unsupported))
    }

    async fn ids(&self) -> io::Result<Vec<String>> {
        if exists(&self.root_dir).is_ok() {
            let mut dirs = read_dir(&self.root_dir).await?;
            let mut result = vec![];
            while let Some(entry) = dirs.next_entry().await? {
                let metadata = entry.metadata().await?;
                if !metadata.file_type().is_file() {
                    continue;
                }
                if let Some(extension) = entry.path().extension()
                    && extension == "session"
                {
                    if let Ok(id) = entry.file_name().into_string() {
                        result.push(id);
                    }
                }
            }
            result.sort();
            return Ok(result);
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
}

pub mod tests;
