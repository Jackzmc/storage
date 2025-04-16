use std::fs;
use std::io::Cursor;
use log::trace;
use rand::rngs::OsRng;
use rand::{rng, Rng, TryRngCore};
use rand::distr::Alphanumeric;
use rocket::http::{ContentType, Status};
use rocket::{response, Request, Response};
use rocket::fs::relative;
use rocket::response::Responder;
use rocket::serde::Serialize;
use rocket_dyn_templates::handlebars::Handlebars;
use rocket_session_store::{Session, SessionError, SessionResult};
use sqlx::Error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use uuid::Uuid;
use crate::models::user::{UserAuthError,};
use crate::SessionData;
use crate::util::ResponseError::DatabaseError;

pub(crate) fn setup_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("warn,rocket=warn,storage_server=trace").into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

pub async fn set_csrf(session: &Session<'_, SessionData>) -> String {
    let token = gen_csrf_token();
    trace!("set_csrf token={}", token);
    let mut sess = session.get().await.expect("failed to get session data")
        .unwrap_or_else(|| SessionData {
            csrf_token: None,
            login: None,
        });
    sess.csrf_token = Some(token.clone());
    session.set(sess).await.unwrap();
    token
}

pub async fn validate_csrf(session: &Session<'_, SessionData>, form_csrf_token: &str) -> Result<bool, SessionError> {
    if let Some(mut sess) = session.get().await? {
        if let Some(sess_token) = sess.csrf_token {
            let success = sess_token == form_csrf_token;
            if success {
                sess.csrf_token = None;
                session.set(sess).await?;
                return Ok(true)
            }
        }
    }
    Ok(false)
}

pub fn gen_csrf_token() -> String {
    rng()
        .sample_iter(&Alphanumeric)
        .map(char::from) // map added here
        .take(30)
        .collect()
}

// pub(crate) fn setup_template_engine() -> Handlebars<'static> {
//     let mut hb = Handlebars::new();
//     #[cfg(debug_assertions)]
//     hb.set_dev_mode(true);
//
//     let templates = fs::read_dir(relative!("templates")).unwrap();
//     let mut ok = true;
//     for file in templates {
//         let file = file.unwrap();
//         if let Err(e) = hb.register_template_file(file.path().to_str().unwrap(), ) {
//             error!(template, path = %path.display(),
//                     "failed to register Handlebars template: {e}");
//
//             ok = false;
//         }
//     }
//
// hb
// }

#[derive(Debug, Clone, Serialize)]
pub struct JsonErrorResponse {
    pub(crate) code: String,
    pub(crate) message: String
}

#[derive(Debug)]
pub enum ResponseError {
    NotFound(JsonErrorResponse),
    GenericError,
    InternalServerError(JsonErrorResponse),
    DatabaseError(JsonErrorResponse),
    AuthError(UserAuthError),
    CSRFError
}

impl ResponseError {
    fn get_http_status(&self) -> Status {
        match self {
            ResponseError::InternalServerError(_) => Status::InternalServerError,
            ResponseError::GenericError => Status::InternalServerError,
            ResponseError::NotFound(_) => Status::NotFound,
            ResponseError::DatabaseError(_) => Status::InternalServerError,
            ResponseError::AuthError(e) => e.get_response_code(),
            ResponseError::CSRFError => Status::Unauthorized,
            _ => Status::BadRequest,
        }
    }

    fn into_res_err(self) -> JsonErrorResponse {
        match self {
            ResponseError::NotFound(e) => e,
            ResponseError::GenericError => {
                JsonErrorResponse {
                    code: "INTERNAL_SERVER_ERROR".to_string(),
                    message: "An unknown error occurred".to_string(),
                }
            },
            ResponseError::InternalServerError(e) => e,
            DatabaseError(e) => e,
            ResponseError::AuthError(e) => e.into_response_err(),
            ResponseError::CSRFError => {
                JsonErrorResponse {
                    code: "CSRF_VALIDATION_FAILED".to_string(),
                    message: "CSRF Token is invalid / expired or does not exist. Reload the form and try again".to_string(),
                }
            }
        }
    }
}
impl From<sqlx::Error> for ResponseError {
    fn from(value: Error) -> Self {
        let err = value.into_database_error().unwrap();
        DatabaseError(JsonErrorResponse {
            code: err.code().map(|s| s.to_string()).unwrap_or_else(|| "UNKNOWN".to_string()),
            message: err.message().to_string(),
        })
    }
}

impl From<UserAuthError> for ResponseError {
    fn from(value: UserAuthError) -> Self {
        ResponseError::AuthError(value)
    }
}

impl std::fmt::Display for ResponseError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "Error {}.", self.get_http_status())
    }
}

impl<'r> Responder<'r, 'static> for ResponseError {
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        // serialize struct into json string
        let status = self.get_http_status();
        let err_response = serde_json::to_string(&self.into_res_err()).unwrap();

        Response::build()
            .status(status)
            .header(ContentType::JSON)
            .sized_body(err_response.len(), Cursor::new(err_response))
            .ok()
    }
}