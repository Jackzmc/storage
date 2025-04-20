use std::error::Error;
use std::fmt::{Display, Formatter};
use bcrypt::BcryptError;
use chrono::NaiveDateTime;
use rocket::form::Context;
use rocket::http::Status;
use rocket::{form, Request};
use rocket::form::error::Entity;
use rocket::response::Responder;
use rocket::serde::Serialize;
use rocket::serde::uuid::Uuid;
use sqlx::{query_as, FromRow};
use crate::consts::{DISABLE_LOGIN_CHECK, ENCRYPTION_ROUNDS};
use crate::{LoginSessionData, SessionData, DB};
use crate::models::repo::RepoModel;
use crate::util::JsonErrorResponse;

#[derive(Serialize, Clone, Debug, FromRow)]
pub struct UserModel {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
    pub name: String
}
#[derive(Serialize, Clone, Debug, FromRow)]
pub struct UserModelWithPassword {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: Option<String>,
    pub created_at: NaiveDateTime,
    pub name: String
}

#[derive(Debug)]
pub enum UserAuthError {
    DatabaseError(sqlx::Error),
    UserNotFound,
    UserAlreadyExists,
    PasswordInvalid,
    EncryptionError(BcryptError),
}

impl Display for UserAuthError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.get_err_code(), self.get_err_msg())
    }
}

impl Error for UserAuthError {

}
impl UserAuthError {
    fn get_err_code(&self) -> String {
        match self {
            UserAuthError::DatabaseError(_) => "DATABASE_ERROR",
            UserAuthError::UserNotFound => "USER_NOT_FOUND",
            UserAuthError::UserAlreadyExists => "USER_EXISTS",
            UserAuthError::PasswordInvalid => "PASSWORD_INVALID",
            UserAuthError::EncryptionError(_) => "ENCRYPTION_ERROR",
        }.to_string()
    }
    fn get_err_msg(&self) -> String {
        match self {
            UserAuthError::DatabaseError(e) => format!("Error from database: {}", e.to_string()),
            UserAuthError::UserNotFound => "No user found with provided username or email".to_string(),
            UserAuthError::UserAlreadyExists => "User already exists".to_string(),
            UserAuthError::PasswordInvalid => "Password is invalid or incorrect".to_string(),
            UserAuthError::EncryptionError(_) => "Error occurred during password encryption".to_string()
        }.to_string()
    }

    pub(crate) fn get_response_code(&self) -> Status {
        match self {
            UserAuthError::DatabaseError(_) => Status::InternalServerError,
            UserAuthError::UserNotFound => Status::NotFound,
            UserAuthError::UserAlreadyExists => Status::Conflict,
            UserAuthError::PasswordInvalid => Status::Unauthorized,
            UserAuthError::EncryptionError(_) => Status::InternalServerError
        }
    }
    pub(crate) fn into_response_err(self) -> JsonErrorResponse {
        JsonErrorResponse {
            code: self.get_err_code(),
            message: self.get_err_msg(),
        }
    }

}


pub async fn get_user(pool: &DB, user_id: &str) -> Result<Option<UserModel>, anyhow::Error> {
    let user_id = Uuid::parse_str(user_id)?;
    query_as!(UserModel, "select id, username, created_at, email, name from storage.users where id = $1", user_id)
        .fetch_optional(pool)
        .await.map_err(anyhow::Error::from)
}
/// Validates user login form, returning Some on success or None (with ctx containing errors) on failure
pub async fn validate_user_form(ctx: &mut Context<'_>, pool: &DB) -> Option<UserModel> {
    let username = ctx.field_value("username").unwrap();
    let password = ctx.field_value("password").unwrap(); // TODO: no unwrap
    match validate_user(pool, username, password).await {
        Ok(u) => Some(u),
        Err(UserAuthError::PasswordInvalid | UserAuthError::UserNotFound) => {
            ctx.push_error(form::Error::validation("Username or password is incorrect").with_entity(Entity::Form));
            None
        },
        Err(e) => {
            ctx.push_error(form::Error::custom(e));
            None
        }
    }

}
pub async fn validate_user(pool: &DB, email_or_usrname: &str, password: &str) -> Result<UserModel, UserAuthError> {
    let user = query_as!(UserModelWithPassword,
        "select id, username, password, created_at, email, name  from storage.users where email = $1 OR username = $1", email_or_usrname
    )
        .fetch_optional(pool)
        .await
        .map_err(|e| UserAuthError::DatabaseError(e))?;
    let Some(user) = user else {
        return Err(UserAuthError::UserNotFound);
    };
    if let Some(db_password) = user.password {
        if !*DISABLE_LOGIN_CHECK || bcrypt::verify(password, &db_password).map_err(|e| UserAuthError::EncryptionError(e))? {
            return Ok(UserModel {
                id: user.id,
                email: user.email,
                username: user.username,
                created_at: user.created_at,
                name: user.name
            })
        }
    }
    Err(UserAuthError::PasswordInvalid)
}

pub struct CreateUserModel {
    pub username: String,
    pub email: String,
    pub password: String,
    pub name: String
}
pub async fn create_user(pool: &DB, user: CreateUserModel) -> Result<UserModel, UserAuthError> {
    let encrypted_pass = bcrypt::hash(user.password, ENCRYPTION_ROUNDS)
        .map_err(|e| UserAuthError::EncryptionError(e))?;
    todo!()
}