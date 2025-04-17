use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use anyhow::Error;
use rocket::response::stream::ReaderStream;
use rocket::serde::Serialize;
use tokio::io::BufStream;
use crate::managers::repos::RepoContainer;
use crate::{models, DB};
use crate::models::library::LibraryModel;
use crate::models::repo::RepoModel;
use crate::storage::{FileEntry, FileType};
use crate::util::{JsonErrorResponse, ResponseError};

pub struct Library {
    model: LibraryModel,
    repo: RepoContainer,
}

impl Library {
    pub fn new(library_model: LibraryModel, repo: RepoContainer) -> Library {
        Library {
            model: library_model,
            repo
        }
    }

    pub fn model(&self) -> &LibraryModel {
        &self.model
    }

    pub async fn get_read_stream(&self, rel_path: &PathBuf) -> Result<BufReader<File>, anyhow::Error> {
        let mut repo = self.repo.read().await;
        repo.backend.get_read_stream(&self.model.id.to_string(), rel_path)
    }

    pub async fn touch_file(&self, rel_path: &PathBuf, file_type: FileType) -> Result<(), anyhow::Error> {
        let mut repo = self.repo.read().await;
        repo.backend.touch_file(&self.model.id.to_string(), rel_path, file_type)
    }

    pub async fn write_file(&self, rel_path: &PathBuf, contents: &[u8]) -> Result<(), anyhow::Error> {
        let mut repo = self.repo.read().await;
        repo.backend.write_file(&self.model.id.to_string(), rel_path, contents)
    }

    pub async fn read_file(&self, rel_path: &PathBuf) -> Result<Option<Vec<u8>>, anyhow::Error> {
        let repo = self.repo.read().await;
        repo.backend.read_file(&self.model.id.to_string(), rel_path)
    }

    pub async fn list_files(&self, rel_path: &PathBuf) -> Result<Vec<FileEntry>, anyhow::Error> {
        let repo = self.repo.read().await;
        repo.backend.list_files(&self.model.id.to_string(), rel_path)
    }

    pub async fn delete_file(&self, rel_path: &PathBuf) -> Result<(), anyhow::Error> {
        let repo = self.repo.read().await;
        repo.backend.delete_file(&self.model.id.to_string(), rel_path)
    }
    pub async fn move_file(&self, rel_path: &PathBuf, new_rel_path: &PathBuf) -> Result<(), Error> {
        let repo = self.repo.read().await;
        repo.backend.move_file(&self.model.id.to_string(), rel_path, new_rel_path)
    }
}