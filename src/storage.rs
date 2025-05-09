mod local;
mod s3;

use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use anyhow::{anyhow, Error};
use int_enum::IntEnum;
use rocket::FromFormField;
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

#[derive(Debug, Serialize, Deserialize, FromFormField, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileType {
    File,
    Folder,
    Symlink,
    Other
}

impl From<std::fs::FileType> for FileType {
    fn from(value: std::fs::FileType) -> Self {
        if value.is_file() {
            FileType::File
        } else if value.is_dir() {
            FileType::Folder
        } else if value.is_symlink() {
            FileType::Symlink
        } else {
            FileType::Other
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    // last_modified:
    pub size: u64,
    #[serde(rename="type")]
    pub _type: FileType,
}



pub fn get_backend(storage_type: &str, settings: &JsonValue) -> Result<Option<Box<dyn StorageBackend + Send + Sync>>, anyhow::Error> {
    Ok(match storage_type {
        "local" => Some(Box::new(LocalStorage::new(settings)?)),
        _ => None
    })
}

pub trait StorageBackend {
    // fn new(settings: &JsonValue) -> Result<Self, StorageBackendError>;
    fn touch_file(&self, library_id: &str, rel_path: &PathBuf, file_type: FileType) -> Result<(), anyhow::Error>;
    fn write_file(&self, library_id: &str, rel_path: &PathBuf, contents: &[u8]) -> Result<(), anyhow::Error>;

    fn read_file(&self, library_id: &str, rel_path: &PathBuf) -> Result<Option<Vec<u8>>, anyhow::Error>;

    fn list_files(&self, library_id: &str, rel_path: &PathBuf) -> Result<Vec<FileEntry>, anyhow::Error>;

    fn delete_file(&self, library_id: &str, rel_path: &PathBuf) -> Result<(), anyhow::Error>;
    fn move_file(&self, library_id: &str, rel_path: &PathBuf, new_rel_path: &PathBuf) -> Result<(), Error>;
    fn get_read_stream(&self, library_id: &str, rel_path: &PathBuf,) -> Result<BufReader<File>, Error>;
}
