mod local;
mod s3;

use std::path::PathBuf;
use anyhow::{anyhow, Error};
use int_enum::IntEnum;
use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::types::JsonValue;
use crate::storage::local::LocalStorage;
use crate::storage::s3::S3Storage;

pub enum StorageBackendMap {
    Local(LocalStorage),
    S3(S3Storage)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub file_name: String,
    // last_modified:
    pub file_size: u64,
}

pub fn get_backend(storage_type: &str, settings: &JsonValue) -> Result<Option<Box<dyn StorageBackend + Send + Sync>>, anyhow::Error> {
    Ok(match storage_type {
        "local" => Some(Box::new(LocalStorage::new(settings)?)),
        _ => None
    })
}

pub trait StorageBackend {
    // fn new(settings: &JsonValue) -> Result<Self, StorageBackendError>;

    fn write_file(&self, library_id: &str, rel_path: &PathBuf, contents: &[u8]) -> Result<(), anyhow::Error>;

    fn read_file(&self, library_id: &str, rel_path: &PathBuf) -> Result<Option<Vec<u8>>, anyhow::Error>;

    fn list_files(&self, library_id: &str, rel_path: &PathBuf) -> Result<Vec<FileEntry>, anyhow::Error>;

    fn delete_file(&self, library_id: &str, rel_path: &PathBuf) -> Result<(), anyhow::Error>;
    fn move_file(&self, library_id: &str, rel_path: &PathBuf, new_rel_path: &PathBuf) -> Result<(), Error>;
}
