// SPDX-FileCopyrightText: 2025 All contributors
//
// SPDX-License-Identifier: GPL-2.0-or-later

//! Storage Modul for the laptimer
//!
//! Provides the interfaces and implementation to store and load session and track data on linux based systems.

use chrono::NaiveDateTime;
use common::{
    session::{Session, SessionInfo},
    track::Track,
};
use module_core::{
    DeleteSessionRequestPtr, DeleteSessionResponsePtr, EmptyRequestPtr, Event, EventKind,
    LoadSessionRequestPtr, LoadSessionResponsePtr, LoadStoredTrackIdsResponsePtr,
    LoadStoredTracksReponsePtr, ModuleCtx, Response, SaveSessionRequestPtr, SaveSessionResponsePtr,
    StoredSessionIdsResponsePtr,
};
use std::{
    fs::{DirBuilder, exists},
    io::{self},
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};
use tokio::{
    fs::read_dir,
    io::{AsyncReadExt, AsyncWriteExt},
};
use tracing::{debug, error, info};

/// A file system–based implementation of a storage.
///
/// This struct is responsible for persisting session and track data as files in a specified root directory.
/// Each session is stored as a separate file with the `.session` extension in the folder session.
/// Each session is track as a separate file with the `.track` extension in the folder track.
///
/// ## Important
///
/// `FilesSystemStorage` **does not implement any internal synchronization or locking mechanisms**.
/// Therefore, **only one instance should be used per `root_dir` in the application at any time**.
/// Creating multiple instances pointing to the same directory may result in data races,
/// file corruption, or unexpected behavior.
pub struct FilesSystemStorage {
    session_root_dir: String,
    track_root_dir: String,
    module_ctx: ModuleCtx,
}

impl FilesSystemStorage {
    pub fn new(root_dir: &PathBuf, ctx: ModuleCtx) -> Self {
        let mut session_file_path = std::path::PathBuf::from(&root_dir);
        session_file_path.push("session");
        let mut track_file_path = PathBuf::from(&root_dir);
        track_file_path.push("track");
        if let Err(e) = DirBuilder::new().recursive(true).create(&session_file_path) {
            error!(
                "Failed to create session dir folder {}. Error: {}",
                session_file_path.to_string_lossy(),
                e
            );
        }
        info!(
            "Using session storage folder: {}",
            session_file_path.to_string_lossy()
        );
        info!(
            "Using track storage folder: {}",
            track_file_path.to_string_lossy()
        );
        FilesSystemStorage {
            session_root_dir: session_file_path.to_string_lossy().to_string(),
            track_root_dir: track_file_path.to_string_lossy().to_string(),
            module_ctx: ctx,
        }
    }

    /// Persists a session and its derived metadata, returning the session `id`.
    ///
    /// Process:
    /// - Acquires a read lock on `session` (recovers inner value if the lock is poisoned).
    /// - Serializes the `Session` to JSON and computes a stable `id`.
    /// - Builds a `SessionInfo` (date/time, track name, lap count) and serializes it to JSON.
    /// - Releases the lock before performing any filesystem I/O.
    /// - Writes both JSON payloads to disk via `save_session` and `save_session_info`.
    ///
    /// Notes:
    /// - Serialization currently happens synchronously on the current thread (see TODOs).
    ///
    /// Returns:
    /// - `Ok(id)` for the saved session identifier.
    ///
    /// Errors:
    /// - Propagates errors from serialization and underlying file I/O operations.
    async fn save(&self, session: &RwLock<Session>) -> std::io::Result<String> {
        let json_session;
        let id;
        let json_session_info;
        {
            let session = session.read().unwrap_or_else(|e| e.into_inner());
            json_session = Session::to_json(&session)?; // TODO! this sould be done async
            id = FilesSystemStorage::get_id(&session);
            let session_info = SessionInfo::new(
                id.clone(),
                NaiveDateTime::new(session.date, session.time),
                session.track.name.clone(),
                session.laps.len(),
            );
            json_session_info = SessionInfo::to_json(&session_info)?; // TODO! this sould be done async
        }
        self.save_session(&id, &json_session).await?;
        self.save_session_info(&id, &json_session_info).await?;
        Ok(id)
    }

    /// Saves the session payload for the given `id`.
    ///
    /// The target file path is resolved via `get_session_file_path(id)`. The file is
    /// created or truncated, the UTF-8 bytes of `session` are written, and the data is
    /// flushed to disk via `sync_all`.
    ///
    /// Errors:
    /// - Propagates I/O errors from file creation, writing, or syncing.
    /// - Returns `io::ErrorKind::NotFound` if the parent directory does not exist.
    async fn save_session(&self, id: &str, session: &str) -> io::Result<()> {
        let file_path = self.get_session_file_path(id);
        self.save_bytes(&file_path, session.as_bytes()).await?;
        Ok(())
    }

    /// Saves the session metadata/info payload for the given `id`.
    ///
    /// The target file path is resolved via `get_session_info_file_path(id)`. The file is
    /// created or truncated, the UTF-8 bytes of `session_info` are written, and the data is
    /// flushed to disk via `sync_all`.
    ///
    /// Errors:
    /// - Propagates I/O errors from file creation, writing, or syncing.
    /// - Returns `io::ErrorKind::NotFound` if the parent directory does not exist.
    async fn save_session_info(&self, id: &str, session_info: &str) -> io::Result<()> {
        let file_path = self.get_session_info_file_path(id);
        self.save_bytes(&file_path, session_info.as_bytes()).await?;
        Ok(())
    }

    /// Writes arbitrary bytes to the file at `path`, ensuring they are persisted.
    ///
    /// The file is created if it does not exist, or truncated if it does. After writing
    /// `data`, the file is explicitly synced to ensure durability.
    ///
    /// Errors:
    /// - Propagates I/O errors from file creation (`tokio::fs::File::create`),
    ///   writing (`AsyncWriteExt::write_all`), and syncing (`File::sync_all`).
    /// - Returns `io::ErrorKind::NotFound` if any parent directory is missing.
    async fn save_bytes(&self, path: &str, data: &[u8]) -> io::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(data).await?;
        file.sync_all().await?;
        Ok(())
    }

    async fn load_file(&self, file_path: &str) -> io::Result<String> {
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut json = String::default();
        file.read_to_string(&mut json).await?;
        Ok(json)
    }

    /// Deletes the `.info` metadata file for the given session `id`.
    ///
    /// The target path is constructed as `<session_root_dir>/<id>.info`. If the file
    /// exists, it is removed asynchronously. If it does not exist, an
    /// `io::ErrorKind::NotFound` is returned.
    ///
    /// Errors:
    /// - Propagates I/O errors from `tokio::fs::remove_file`.
    /// - May return `io::ErrorKind::NotFound` if the file is absent.
    async fn delete_info(&self, id: &str) -> io::Result<()> {
        let mut file_path = std::path::PathBuf::from(&self.session_root_dir);
        file_path.push(id);
        file_path.set_extension("info");
        if exists(&file_path).is_ok() {
            tokio::fs::remove_file(file_path).await?;
            return Ok(());
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }

    async fn delete(&self, id: &str) -> io::Result<()> {
        let file_path = self.get_session_file_path(id);
        if exists(&file_path).is_ok() {
            tokio::fs::remove_file(file_path).await?;
            return Ok(());
        }
        Err(io::Error::from(io::ErrorKind::NotFound))
    }

    /// Load all persisted `SessionInfo` entries from the session root directory.
    ///
    /// Behavior:
    /// - Scans `self.session_root_dir` for files with the `.info` extension.
    /// - Reads each file, deserializes JSON into `SessionInfo`, and collects valid entries.
    /// - Logs and skips files that fail to load or parse; non-file entries are ignored.
    /// - Sorts the resulting list by `id` in ascending order.
    ///
    /// Returns:
    /// - `Ok(Arc<Vec<SessionInfo>>)` on success (possibly an empty vector).
    /// - `Err(io::ErrorKind::NotFound)` if the session folder is missing.
    /// - Other `io::Error`s for unexpected I/O failures.
    async fn load_session_infos(&self) -> io::Result<Arc<Vec<SessionInfo>>> {
        if exists(&self.session_root_dir).is_ok() {
            let mut dirs = read_dir(&self.session_root_dir).await?;
            let mut infos = Vec::<SessionInfo>::new();
            while let Some(entry) = dirs.next_entry().await? {
                let metadata = entry.metadata().await?;
                if !metadata.file_type().is_file() {
                    continue;
                }
                if let Some(ext) = entry.path().extension()
                    && ext == "info"
                    && let Some(id) = entry.path().file_stem()
                {
                    let file_path = entry.path().to_string_lossy().to_string();
                    match self.load_file(&file_path).await {
                        Ok(json) => match SessionInfo::from_json(&json) {
                            Ok(info) => {
                                debug!(
                                    "Loaded session info with id {} from file {}",
                                    id.to_string_lossy().to_string(),
                                    file_path
                                );
                                infos.push(info);
                            }
                            Err(e) => {
                                error!(
                                    "Failed to parse session info from file {}. Error: {}",
                                    file_path, e
                                );
                                continue;
                            }
                        },
                        Err(e) => {
                            error!(
                                "Failed to load session info from file {}. Error: {}",
                                file_path, e
                            );
                            continue;
                        }
                    }
                }
            }
            infos.sort_by(|a, b| a.id.cmp(&b.id));
            return Ok(Arc::new(infos));
        }
        error!("Not session folder found in {}", self.session_root_dir);
        Err(io::Error::from(io::ErrorKind::NotFound))
    }

    async fn ids(&self, dir: &str, extension: &str) -> io::Result<Vec<String>> {
        if exists(dir).is_ok() {
            let mut dirs = read_dir(dir).await?;
            let mut result = vec![];
            while let Some(entry) = dirs.next_entry().await? {
                let metadata = entry.metadata().await?;
                if !metadata.file_type().is_file() {
                    continue;
                }
                if let Some(ext) = entry.path().extension()
                    && ext == extension
                    && let Some(id) = entry.path().file_stem()
                {
                    debug!(
                        "Found file with id {} in folder {}",
                        id.to_string_lossy().to_string(),
                        dir
                    );
                    result.push(id.to_string_lossy().to_string());
                }
            }
            result.sort();
            return Ok(result);
        }
        error!("Not folder found in {}", self.session_root_dir);
        Err(io::Error::from(io::ErrorKind::NotFound))
    }

    /// Handle a request to load stored session identifiers and reply with the result.
    ///
    /// Behavior:
    /// - Attempts to load all persisted `SessionInfo` entries via `load_session_infos`.
    /// - On success, returns the loaded infos; on failure, responds with an empty list.
    /// - Emits `EventKind::LoadStoredSessionIdsResponseEvent` back to the requester.
    ///
    /// The response mirrors the original request id and sender address.
    async fn handle_load_stored_ids_request(&self, req: &EmptyRequestPtr) {
        let infos = match self.load_session_infos().await {
            Ok(infos) => {
                debug!(
                    "Load session infos {:?} from {}",
                    infos, self.session_root_dir
                );
                infos
            }
            Err(_) => Arc::new(vec![]),
        };
        let resp = StoredSessionIdsResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data: infos,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::LoadStoredSessionIdsResponseEvent(resp),
        });
    }

    async fn handle_save_request(&self, req: &SaveSessionRequestPtr) {
        let result = self.save(&req.data).await;
        let data = match result {
            Ok(id) => {
                debug!("Stored session with id {} in {}", id, self.session_root_dir);
                Ok(id)
            }
            Err(e) => {
                debug!(
                    "Failed to store session with id {} in {}. Error:{}",
                    req.data.read().unwrap_or_else(|e| e.into_inner()).id,
                    self.session_root_dir,
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
        let file_path = self.file_path(&req.data, Path::new(&self.session_root_dir), "session");
        let data = match self
            .load_file(&file_path)
            .await
            .and_then(|json| Session::from_json(&json).map_err(|e| e.into()))
        {
            Ok(session) => {
                debug!("Load session with filename {}", file_path);
                Ok(Arc::new(RwLock::new(session)))
            }
            Err(e) => {
                debug!(
                    "Failed to load session with filename {}. Error: {}",
                    file_path, e
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

    /// Handle a delete-session request and emit a response event.
    ///
    /// Workflow:
    /// - Extract the session `id` from the request.
    /// - Attempt to delete the session info/metadata first.
    /// - If that succeeds, attempt to delete the session data itself.
    /// - Build and send a `DeleteSessionResponseEvent` containing the outcome.
    ///
    /// The response echoes the original request id and sender address, and carries
    /// the first encountered error (if any) as its data.
    async fn handle_delete_request(&self, req: &DeleteSessionRequestPtr) {
        let id = &req.data;
        let mut result = self.delete_info(id).await.map_err(|e| e.kind());
        if result.is_ok() {
            result = self.delete(id).await.or(result);
        }
        let resp = DeleteSessionResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data: result,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::DeleteSessionResponseEvent(resp),
        });
    }

    async fn handle_load_stored_track_ids_request(&self, req: &EmptyRequestPtr) {
        let ids = self.ids(&self.track_root_dir, "track").await;
        let data = match ids {
            Ok(ids) => {
                debug!("Load track ids {:?} from {}", ids, self.track_root_dir);
                ids
            }
            Err(_) => vec![],
        };

        let resp = LoadStoredTrackIdsResponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data,
        });
        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::LoadStoredTrackIdsResponseEvent(resp),
        });
    }

    async fn handle_all_load_stored_track_request(&self, req: &EmptyRequestPtr) {
        let mut tracks: Vec<Track> = vec![];
        if let Ok(ids) = self.ids(&self.track_root_dir, "track").await {
            for id in ids.iter() {
                let file_path = self.file_path(id, Path::new(&self.track_root_dir), "track");
                match self
                    .load_file(&file_path)
                    .await
                    .and_then(|json| Track::from_json(&json).map_err(|e| e.into()))
                {
                    Ok(track) => {
                        debug!("Load track from \"{file_path}\".");
                        tracks.push(track);
                    }
                    Err(e) => {
                        error!("Failed to load track \"{file_path}\". Error: {e}");
                        continue;
                    }
                }
            }
        }

        let resp = LoadStoredTracksReponsePtr::new(Response {
            id: req.id,
            receiver_addr: req.sender_addr,
            data: tracks,
        });

        let _ = self.module_ctx.sender.send(Event {
            kind: EventKind::LoadAllStoredTracksResponseEvent(resp),
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
        let mut file_path = std::path::PathBuf::from(&self.session_root_dir);
        file_path.push(id);
        file_path.set_extension("session");
        file_path.to_string_lossy().to_string()
    }

    /// Build the absolute path to the session info file for the given session `id`.
    ///
    /// The path is constructed as: `<session_root_dir>/<id>.info`.
    /// Returns the path as an owned `String` (via lossy conversion from `OsStr`).
    fn get_session_info_file_path(&self, id: &str) -> String {
        let mut file_path = std::path::PathBuf::from(&self.session_root_dir);
        file_path.push(id);
        file_path.set_extension("info");
        file_path.to_string_lossy().to_string()
    }

    fn file_path(&self, id: &str, path: &Path, extension: &str) -> String {
        let mut file_path = std::path::PathBuf::from(path);
        file_path.push(id);
        file_path.set_extension(extension);
        file_path.to_string_lossy().to_string()
    }
}

#[async_trait::async_trait]
impl module_core::Module for FilesSystemStorage {
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
                                },
                                EventKind::LoadSessionRequestEvent(request) => {
                                    self.handle_load_request(&request).await;
                                },
                                EventKind::DeleteSessionRequestEvent(request) => {
                                    self.handle_delete_request(&request).await;
                                },
                                EventKind::LoadStoredTrackIdsRequest(request) => {
                                    self.handle_load_stored_track_ids_request(&request).await;
                                }
                                EventKind::LoadAllStoredTracksRequestEvent(request) => {
                                    self.handle_all_load_stored_track_request(&request).await;
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
