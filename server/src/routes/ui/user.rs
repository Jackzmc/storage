use std::path::PathBuf;
use std::sync::Arc;
use rocket::{get, State};
use rocket::serde::json::Json;
use rocket_dyn_templates::{context, Template};
use tokio::sync::Mutex;
use crate::managers::libraries::LibraryManager;
use crate::util::{JsonErrorResponse, ResponseError};

#[get("/")]
pub async fn index() -> Template {
    Template::render("index", context! { test: "value" })
}

#[get("/libraries/<library_id>/<_>/<path..>")]
pub async fn list_library_files(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: PathBuf) -> Result<Template, ResponseError> {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    let files = library.list_files(&PathBuf::from(path)).await
        .map_err(|e| ResponseError::InternalServerError(JsonErrorResponse {
            code: "STORAGE_ERROR".to_string(),
            message: e.to_string(),
        }))?;

    Ok(Template::render("libraries", context! {
        library: library.model(),
        files: files
    }))
}