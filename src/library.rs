use std::collections::HashMap;
use crate::{models, DB};
use crate::objs::repo::{Repo};
use crate::storage::StorageBackend;
use crate::user::User;

pub struct Library {
    pub repo: Repo,
    pub name: String,
    pub owner: User,
    // permissions:
}

// pub async fn get_library(pool: &DB, library_id: &str) -> Result<Option<Library>, anyhow::Error> {
//     let library = models::library::get_library(pool, library_id)?;
//     let Some(library) = library else {
//         return Ok(None)
//     };
//     let repo = get_repo(pool, &library.repo_id).await?
//         .ok_or(anyhow::Error::msg("Repository not found."))?;
//     let owner = get_owner(pool, &library.owner_id).await?
//         .ok_or(anyhow::Error::msg("Owner not found."))?;
//     Ok(Some(Library {
//         repo: repo,
//         name: library.name,
//         owner: (),
//     }))
// }