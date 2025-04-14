use anyhow::anyhow;
use chrono::NaiveDateTime;
use rocket::serde::{Serialize, Deserialize};
use rocket::time::Date;
use sqlx::{query_as};
use sqlx::types::{Uuid};
use crate::{models, DB};
use crate::library::Library;
use crate::models::repo::RepoModel;
use crate::objs::repo::{Repo};
use crate::user::User;

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryModel {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub repo_id: String,
    pub created_at: NaiveDateTime,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LibraryWithRepoModel {
    pub library: LibraryModel,
    pub storage_type: String,
}

pub async fn get_library(pool: &DB, library_id: &str) -> Result<Option<LibraryModel>, anyhow::Error> {
    let library_id = Uuid::parse_str(library_id)?;
    let library = query_as!(LibraryModel, "select * from storage.libraries where id = $1", library_id)
        .fetch_optional(pool)
        .await.map_err(anyhow::Error::from)?;
    // if library.is_none() { return Ok(None) }
    Ok(library)
}

pub async fn get_library_with_repo(pool: &DB, library_id: &str) -> Result<Option<LibraryWithRepoModel>, anyhow::Error> {
    let Some(library) = get_library(pool, library_id).await? else {
        return Ok(None)
    };
    let repo = models::repo::get_repo(pool, &library.repo_id).await?
        .ok_or_else(|| anyhow!("Repository does not exist"))?;
    Ok(Some(LibraryWithRepoModel {
        storage_type: repo.storage_type,
        library: library
    }))
}