use chrono::NaiveDateTime;
use rocket::serde::{Deserialize, Serialize};
use sqlx::{query_as, Value};
use sqlx::types::{Json, JsonValue, Uuid};
use crate::DB;
use crate::models::library::LibraryModel;

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct RepoModel {
    pub id: String,
    pub created_at: NaiveDateTime,
    pub storage_type: String,
    pub storage_settings: Json<JsonValue>, // for now
    pub flags: i16,
}

pub async fn get_repo(pool: &DB, repo_id: &str) -> Result<Option<RepoModel>, anyhow::Error> {
    query_as!(RepoModel, "select * from storage.repos where id = $1", repo_id)
        .fetch_optional(pool)
        .await.map_err(anyhow::Error::from)
}