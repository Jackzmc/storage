use std::io::Cursor;
use rocket::http::{ContentType, Status};
use rocket::{response, Request, Response};
use rocket::response::Responder;
use rocket::serde::Serialize;
use sqlx::Error;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::util::ResponseError::DatabaseError;

pub(crate) fn setup_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::filter::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=trace,storage-server=trace", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

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
}

impl ResponseError {
    fn get_http_status(&self) -> Status {
        match self {
            ResponseError::InternalServerError(_) => Status::InternalServerError,
            ResponseError::GenericError => Status::InternalServerError,
            ResponseError::NotFound(_) => Status::NotFound,
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