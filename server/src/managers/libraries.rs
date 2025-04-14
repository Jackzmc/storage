use std::collections::HashMap;
use sqlx::{Pool, Postgres};
use tokio::sync::RwLock;
use crate::objs::library::Library;
use crate::managers::repos::{RepoContainer, RepoManager};
use crate::models;
use crate::util::{JsonErrorResponse, ResponseError};

pub struct LibraryManager {
    pool: Pool<Postgres>,
    repos: RepoManager // TODO: make this rwlock so repo manager itself can be clone?
}

impl LibraryManager {
    pub fn new(pool: Pool<Postgres>, repos: RepoManager) -> Self {
        Self {
            pool,
            repos
        }
    }

    pub async fn get(&self, library_id: &str) -> Result<Library, ResponseError> {
        let Some(library) = models::library::get_library(&self.pool, library_id).await
            .map_err(|e| ResponseError::GenericError)? else {
            return Err(ResponseError::NotFound(JsonErrorResponse {
                code: "LIBRARY_NOT_FOUND".to_string(),
                message: "Library could not be found".to_string()
            }))
        };
        let Some(repo) = self.repos.get_repo(&library.repo_id).await else {
            return Err(ResponseError::NotFound(JsonErrorResponse {
                code: "LIBRARY_INVALID_REPO".to_string(),
                message: "Library is incorrectly configured, repository does not exist".to_string()
            }))
        };
        Ok(Library::new(library, repo))
    }
}