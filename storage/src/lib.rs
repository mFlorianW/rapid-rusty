//! Storage Modul for the laptimer
//!
//! Provides the interfaces and implementation to store and load session and track data on linux based systems.

use async_trait::async_trait;
use common::session::Session;
use std::{fs::exists, io};
use tokio::{
    fs::read_dir,
    io::{AsyncReadExt, AsyncWriteExt},
};

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
    /// If a session with the same ID already exists, it will be **overwritten**.
    ///
    /// # Arguments
    /// * `session` - A reference to the session to be stored.
    ///
    /// # Returns
    /// * `Ok(String)` - The ID under which the session was saved.
    /// * `Err(io::Error)` - If saving fails due to I/O or storage errors.
    async fn save(&self, session: &Session) -> io::Result<String>;

    /// Loads a [`Session`] by its unique ID.
    ///
    /// # Arguments
    /// * `id` - The identifier of the session to load.
    ///
    /// # Returns
    /// * `Ok(Session)` - The deserialized session.
    /// * `Err(io::Error)` - If the session does not exist or cannot be read.
    async fn load(&self, id: &str) -> io::Result<Session>;

    /// Deletes a stored session with the specified ID.
    ///
    /// # Arguments
    /// * `id` - The ID of the session to remove.
    ///
    /// # Returns
    /// * `Ok(())` - If the session was successfully deleted.
    /// * `Err(io::Error)` - If the session could not be deleted or was not found.
    async fn delete(&self, id: &str) -> io::Result<()>;

    /// Returns a list of all stored session IDs.
    ///
    /// This can be used to enumerate all available sessions, e.g., for displaying in a UI.
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - A list of all session IDs.
    /// * `Err(io::Error)` - If an error occurs during listing.
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
    async fn save(&self, session: &Session) -> io::Result<String> {
        let json_session = Session::to_json(session)?;
        let id = SessionFsStorage::get_id(session);
        let file_path = self.get_session_file_path(&id);
        let mut file = tokio::fs::File::create(&file_path).await?;
        file.write_all(json_session.as_bytes()).await?;
        file.sync_all().await?;
        Ok(id)
    }

    async fn load(&self, id: &str) -> io::Result<Session> {
        let file_path = self.get_session_file_path(id);
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut json_session = String::default();
        file.read_to_string(&mut json_session).await?;
        Session::from_json(&json_session).map_err(|_| io::Error::from(io::ErrorKind::Unsupported))
    }

    async fn delete(&self, id: &str) -> io::Result<()> {
        let file_path = self.get_session_file_path(id);
        if exists(&file_path).is_ok() {
            tokio::fs::remove_file(file_path).await?;
            return Ok(());
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
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
                    if let Some(id) = entry.path().file_stem() {
                        result.push(id.to_string_lossy().to_string());
                    }
                }
            }
            result.sort();
            return Ok(result);
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }
}

impl SessionFsStorage {
    /// Returns the unique identifier of the session.
    ///
    /// This method consumes the `Session` instance and returns its `id` as a `String`.
    /// Typically, the ID is used to identify and retrieve sessions from storage.
    ///
    /// # Returns
    /// A `String` containing the session's unique identifier.
    fn get_id(session: &Session) -> String {
        format!(
            "{}_{}_{}",
            session.track.name.to_lowercase(),
            session.date.format("%d_%m_%Y"),
            session.time.format("%H_%M_%S_%3f")
        )
    }

    /// Constructs the full file path for a session based on its ID.
    ///
    /// This function generates a platform-independent path to a session file by:
    /// - Starting from the root directory specified in `self.root_dir`,
    /// - Appending the given `id` as the file name,
    /// - And setting the file extension to `.session`.
    ///
    /// The resulting path is returned as a `String`. It uses a lossy UTF-8 conversion
    /// in case the underlying path contains invalid UTF-8 sequences.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice representing the session identifier.
    ///
    /// # Returns
    ///
    /// A `String` containing the complete file path to the session file.
    fn get_session_file_path(&self, id: &str) -> String {
        let mut file_path = std::path::PathBuf::from(&self.root_dir);
        file_path.push(id);
        file_path.set_extension("session");
        file_path.to_string_lossy().to_string()
    }
}

pub mod tests;
