use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::IpAddr;
use std::sync::Arc;
use anyhow::anyhow;
use rocket::futures::TryStreamExt;
use rocket::serde::Serialize;
use rocket::State;
use rocket_session_store::{Session, SessionStore, Store};
use rocket_session_store::memory::MemoryStore;
use sqlx::{query, query_as, Pool, QueryBuilder};
use uuid::Uuid;
use crate::config::AppConfig;
use crate::consts::{DISABLE_LOGIN_CHECK, ENCRYPTION_ROUNDS};
use crate::{LoginSessionData, SessionData, DB};
use crate::models::user::{UserAuthError, UserModel, UserModelWithPassword};

pub struct UserManager {
    pool: DB,
}

#[derive(Debug, Serialize)]
pub struct CreateUserOptions {
    pub email: String,
    pub username: String,
    pub name: Option<String>,
}

pub type UsersState = UserManager;


pub enum FindUserOption {
    Id(String),
    Email(String),
    Username(String),
}

#[derive(Hash)]
pub struct SSOData {
    pub(crate) provider_id: String,
    pub(crate) sub: String
}

impl UserManager {
    pub fn new(pool: DB) -> Self {
        Self {
            pool,
        }
    }
    pub fn generate_id(sso_data: Option<SSOData>) -> String {
        if let Some(sso_data) = sso_data {
            let mut s = DefaultHasher::new();
            sso_data.hash(&mut s);
            format!("{:x}", s.finish())
        } else {
            uuid::Uuid::new_v4().to_string()
        }
    }

    pub async fn fetch_user(&self, search_options: &[FindUserOption]) -> Result<Option<UserModel>, anyhow::Error> {
        if search_options.is_empty() { return Err(anyhow!("At least one search option must be included"))}
        let mut query = QueryBuilder::new("select id, username, password, created_at, email, name from storage.users where ");
        for option in search_options {
            match option {
                FindUserOption::Id(id) => {
                    query.push("id = $1");
                    query.push_bind(id);
                },
                FindUserOption::Email(email) => {
                    query.push("email = $1");
                    query.push_bind(email);
                }
                FindUserOption::Username(username) => {
                    query.push("username = $1");
                    query.push_bind(username);
                }
            };
        }
        query.build_query_as::<UserModel>()
                .fetch_optional(&self.pool)
                .await
                .map_err(|e| anyhow!(e))
    }
    /// Returns user's id
    pub async fn create_normal_user(&self, user: CreateUserOptions, plain_password: String) -> Result<UserModel, anyhow::Error> {
        let password = bcrypt::hash(plain_password, ENCRYPTION_ROUNDS)
            .map_err(|e| anyhow!(e))?;
        let id = Self::generate_id(None);
        self.create_user(id, user, Some(password)).await
    }
    /// Returns user's id
    pub async fn create_sso_user(&self, user: CreateUserOptions, id: String) -> Result<UserModel, anyhow::Error> {
        self.create_user(id, user, None).await
    }
    async fn create_user(&self, id: String, user: CreateUserOptions, encrypted_password: Option<String>) -> Result<UserModel, anyhow::Error> {
        query!(
            "INSERT INTO storage.users (id, name, password, email, username) VALUES ($1, $2, $3, $4, $5)",
            id,
            user.name,
            encrypted_password,
            user.email,
            user.username
        )
            .execute(&self.pool)
            .await?;
        Ok(UserModel {
            id,
            username: user.username,
            email: user.email,
            created_at: Default::default(),
            name: user.name,
        })
    }

    pub async fn login_user_session(&self, user: UserModel, ip_address: IpAddr, sessions: &Session<'_, SessionData>) {
        sessions.set(SessionData {
            csrf_token: None,
            login: Some(LoginSessionData {
                user,
                ip_address
            }),
        }).await.unwrap();
    }

    pub async fn login_normal_user(&self, email_or_usrname: &str, password: &str, ip: IpAddr, session: &Session<'_, SessionData>) -> Result<UserModel, UserAuthError> {
        let user = query_as!(UserModelWithPassword,
        "select id, username, password, created_at, email, name  from storage.users where email = $1 OR username = $1", email_or_usrname
    )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| UserAuthError::DatabaseError(e))?;
        let Some(user) = user else {
            return Err(UserAuthError::UserNotFound);
        };
        if let Some(db_password) = user.password {
            if !*DISABLE_LOGIN_CHECK || bcrypt::verify(password, &db_password).map_err(|e| UserAuthError::EncryptionError(e))? {
                let model = UserModel {
                    id: user.id,
                    email: user.email,
                    username: user.username,
                    created_at: user.created_at,
                    name: user.name
                };
                self.login_user_session(model.clone(), ip, session).await;
                return Ok(model)
            }
        }
        Err(UserAuthError::PasswordInvalid)
    }
}