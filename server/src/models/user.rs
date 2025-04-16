use chrono::NaiveDateTime;
use rocket::serde::Serialize;
use rocket::serde::uuid::Uuid;
use sqlx::query_as;
use crate::DB;
use crate::models::repo::RepoModel;

#[derive(Serialize, Clone, Debug)]
pub struct UserModel {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub name: String
}

pub async fn get_user(pool: &DB, user_id: &str) -> Result<Option<UserModel>, anyhow::Error> {
    let user_id = Uuid::parse_str(user_id)?;
    query_as!(UserModel, "select id, created_at, name from storage.users where id = $1", user_id)
        .fetch_optional(pool)
        .await.map_err(anyhow::Error::from)
}