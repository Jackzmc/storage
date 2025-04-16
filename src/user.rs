use std::collections::HashMap;
use rocket::serde::uuid::Uuid;
use sqlx::query_as;
use crate::DB;
use crate::library::Library;
use crate::models::user::UserModel;

pub struct User {
    libraries: HashMap<String, Library>,
}

impl User {
    pub fn _idk() -> Self {
        Self {
            libraries: HashMap::new()
        }
    }

    pub fn get_library(&self, id: &str) -> Option<&Library> {
        self.libraries.get(id)
    }
}

