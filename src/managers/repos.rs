use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use sqlx::{PgPool, Pool, Postgres};
use tokio::sync::RwLock;
use crate::{models, DB};
use crate::models::repo::RepoModel;
use crate::objs::repo::Repo;
use crate::util::{JsonErrorResponse, ResponseError};

#[derive(Clone)]
pub struct RepoManager {
    pool: Pool<Postgres>,
    repos: Arc<RwLock<HashMap<String, RepoContainer>>>
}

pub type RepoContainer = Arc<RwLock<Repo>>;

impl RepoManager {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self {
            pool,
            repos: Arc::new(RwLock::new(HashMap::new()))
        }
    }
    pub async fn fetch_repos(&mut self) -> Result<(), anyhow::Error> {
        let repos = sqlx::query_as!(RepoModel, "SELECT * from storage.repos")
            .fetch_all(&self.pool)
            .await.map_err(anyhow::Error::msg)?;
        let mut hashmap = self.repos.write().await;
        for repo in repos.into_iter() {
            let repo = Repo::new(repo);
            let id = repo.id.to_string();
            let repo: RepoContainer = Arc::new(RwLock::new(repo));
            hashmap.insert(id, repo);
        }
        Ok(())
    }
    pub async fn get_repo(&self, id: &str) -> Option<RepoContainer> {
        self.repos.read().await.get(id).cloned()
    }
    pub async fn get_repo_from_library(&self, library_id: &str) -> Result<RepoContainer, ResponseError> {
        let Some(library) = models::library::get_library_with_repo(&self.pool, library_id).await
            .map_err(|e| ResponseError::GenericError)? else {
            return Err(ResponseError::NotFound(JsonErrorResponse {
                code: "LIBRARY_NOT_FOUND".to_string(),
                message: "Library could not be found".to_string()
            }))
        };
        let Some(repo) = self.get_repo(&library.library.repo_id).await else {
            return Err(ResponseError::NotFound(JsonErrorResponse {
                code: "LIBRARY_INVALID_REPO".to_string(),
                message: "Library is incorrectly configured, repository does not exist".to_string()
            }))
        };
        Ok(repo)
    }
}