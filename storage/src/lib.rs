//! Storage Modul for the laptimer
//!
//! Provides the interfaces and implementation to store and load session and track data on linux based systems.

use async_trait::async_trait;
use common::session::{self, Session};
use module_core::{
    DeleteSessionRequestPtr, DeleteSessionResponsePtr, EmptyRequestPtr, Event, EventKind,
    LoadSessionRequestPtr, LoadSessionResponsePtr, ModuleCtx, Response, SaveSessionRequestPtr,
    SaveSessionResponsePtr, StoredSessionIdsResponsePtr,
};
use std::{
    fs::exists,
    io::{self, ErrorKind},
    sync::{Arc, RwLock},
};
use tokio::{
    fs::read_dir,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, error};

/// A file system–based implementation of a session storage.
///
/// This struct is responsible for persisting session data as files in a specified root directory.
/// Each session is stored as a separate file with the `.session` extension.
///
/// ## Important
///
/// `SessionFsStorage` **does not implement any internal synchronization or locking mechanisms**.
/// Therefore, **only one instance should be used per `root_dir` in the application at any time**.
/// Creating multiple instances pointing to the same directory may result in data races,
/// file corruption, or unexpected behavior.
pub struct SessionFsStorage {
    root_dir: String,
    module_ctx: ModuleCtx,
}

impl SessionFsStorage {
    pub fn new(root_dir: &str, ctx: ModuleCtx) -> Self {
        SessionFsStorage {
            root_dir: root_dir.to_string(),
            module_ctx: ctx,
        }
    }

    async fn save(&self, session: &RwLock<Session>) -> std::io::Result<String> {
        let json_session;
        let id;
        {
            let session = session.read().unwrap_or_else(|e| e.into_inner());
            json_session = Session::to_json(&session)?; // TODO! this sould be done async
            id = SessionFsStorage::get_id(&session);
        }
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
                    && let Some(id) = entry.path().file_stem()
                {
                    debug!(
                        "Found session with id {} in folder {}",
                        id.to_string_lossy().to_string(),
                        self.root_dir
                    );
                    result.push(id.to_string_lossy().to_string());
                }
            }
            result.sort();
            return Ok(result);
        }
        error!("Not session folder found in {}", self.root_dir);
        Err(io::Error::from(io::ErrorKind::NotFound))
    }

    async fn handle_load_stored_ids_request(&self, req: &EmptyRequestPtr) {
        let ids = self.ids().await;
        let data = match ids {
            Ok(ids) => {
                debug!("Load session ids {:?} from {}", ids, self.root_dir);
                std::sync::Arc::new(ids)
            }
            Err(_) => std::sync::Arc::new(vec![]),
        };
        let resp = StoredSessionIdsResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::LoadStoredSessionIdsResponseEvent(resp),
        });
    }

    async fn handle_save_request(&self, req: &SaveSessionRequestPtr) {
        let result = self.save(&req.data).await;
        let data = match result {
            Ok(id) => {
                debug!("Stored session with id {} in {}", id, self.root_dir);
                Ok(id)
            }
            Err(e) => {
                debug!(
                    "Failed to store session with id {} in {}. Error:{}",
                    req.data.read().unwrap_or_else(|e| e.into_inner()).id,
                    self.root_dir,
                    e
                );
                Err(e.kind())
            }
        };

        let resp = SaveSessionResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::SaveSessionResponseEvent(resp),
        });
    }

    async fn handle_load_request(&self, req: &LoadSessionRequestPtr) {
        let id = &req.data;
        let load_result = self.load(id).await;
        let data = match load_result {
            Ok(session) => {
                debug!("Delete session with id {} in {}", id, self.root_dir);
                Ok(RwLock::new(session))
            }
            Err(e) => {
                debug!(
                    "Failed to delete session with id {} in {}. Error: {}",
                    id, self.root_dir, e
                );
                Err(e.kind())
            }
        };

        let resp = LoadSessionResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::LoadSessionResponseEvent(resp),
        });
    }

    async fn handle_delete_request(&self, req: &DeleteSessionRequestPtr) {
        let id = &req.data;
        let delete_result = self.delete(id).await;
        let data = match delete_result {
            Ok(_) => {
                debug!("Deleted session with id {} in {}", id, self.root_dir);
                Ok(())
            }
            Err(e) => {
                debug!(
                    "Failed to delete session with id {} in {}",
                    id, self.root_dir
                );
                Err(e.kind())
            }
        };

        let resp = DeleteSessionResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::DeleteSessionResponseEvent(resp),
        });
    }

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

#[async_trait::async_trait]
impl module_core::Module for SessionFsStorage {
    async fn run(&mut self) -> Result<(), ()> {
        let mut run = true;
        while run {
            tokio::select! {
                event = self.module_ctx.receiver.recv() => {
                    match event {
                        Ok(event) => {
                            match event.kind {
                                EventKind::QuitEvent => run = false,
                                EventKind::LoadStoredSessionIdsRequestEvent(request) => {
                                    self.handle_load_stored_ids_request(&request).await;
                                },
                                EventKind::SaveSessionRequestEvent(request) => {
                                    self.handle_save_request(&request).await;
                                }
                                EventKind::LoadSessionRequestEvent(request) => {
                                    self.handle_load_request(&request).await;
                                }
                                EventKind::DeleteSessionRequestEvent(request) => {
                                    self.handle_delete_request(&request).await;
                                }
                                _ => ()
                            }
                        }
                        Err(e) => println!("Error: {}", e),
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests;
