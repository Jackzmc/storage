use std::path::PathBuf;
use chrono::NaiveDateTime;
use sqlx::query_as;
use sqlx::types::{Json, JsonValue};
use crate::{models, DB};
use crate::managers::repos::RepoContainer;
use crate::models::repo::RepoModel;
use crate::storage::{get_backend, StorageBackend};
use crate::util::{JsonErrorResponse, ResponseError};

pub enum RepoFlags {
    None = 0,
    UserAddable = 1
}
pub struct Repo {
    pub id: String,
    pub created_at: NaiveDateTime,
    pub storage_type: String,
    pub storage_settings: Json<JsonValue>, // for now
    pub flags: i16,

    pub backend: Box<dyn StorageBackend + Send + Sync>,
}

impl Repo {
    pub fn new(model: RepoModel) -> Self {
        let backend = get_backend(&model.storage_type, &model.storage_settings.0).unwrap().expect("unknown backend");
        Repo {
            id: model.id,
            created_at: model.created_at,
            storage_type: model.storage_type,
            storage_settings: model.storage_settings,
            flags: model.flags,
            backend
        }
    }
}

