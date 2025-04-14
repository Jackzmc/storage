use std::path::PathBuf;
use std::sync::Arc;
use log::debug;
use rocket::{get, post, Data, State};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use sqlx::{query, Postgres};
use sqlx::types::{Uuid};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use crate::{library, models, DB, MAX_UPLOAD_SIZE};
use crate::managers::libraries::LibraryManager;
use crate::managers::repos::RepoManager;
use crate::models::library::{LibraryModel, LibraryWithRepoModel};
use crate::models::user;
use crate::storage::FileEntry;
use crate::util::{JsonErrorResponse, ResponseError};

#[get("/")]
pub(crate) fn index() -> &'static str {
    "Hello, world!"
}


#[get("/library/<library_id>")]
pub(crate) async fn get_library(pool: &State<DB>, library_id: &str) -> Result<Option<Json<LibraryWithRepoModel>>, ResponseError> {
    let library = models::library::get_library_with_repo(pool, library_id).await
        .map_err(|e| ResponseError::GenericError)?;
    debug!("{:?}", library);
    Ok(library.map(|lib| Json(lib)))
}

#[get("/library/<library_id>/files?<path>")]
pub(crate) async fn list_library_files(pool: &State<DB>, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str) -> Result<Json<Vec<FileEntry>>, ResponseError> {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    library.list_files(&PathBuf::from(path)).await
        .map(|files| Json(files))
        .map_err(|e| ResponseError::InternalServerError(JsonErrorResponse {
            code: "STORAGE_ERROR".to_string(),
            message: e.to_string(),
        }))
}

#[get("/library/<library_id>/files/download?<path>")]
pub(crate) async fn get_library_file(pool: &State<DB>, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str) -> Result<Vec<u8>, ResponseError>   {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    match library.read_file(&PathBuf::from(path)).await
        .map_err(|e| ResponseError::GenericError)?
    {
        None => {
            Err(ResponseError::NotFound(JsonErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: "Requested file does not exist".to_string()
            }))
        }
        Some(contents) => {
            // TODO: headers?
            Ok(contents)
        }
    }
}

#[post("/library/<library_id>/files/move?<from>&<to>")]
pub(crate) async fn move_library_file(pool: &State<DB>, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, from: &str, to: &str) -> Result<(), ResponseError>   {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    library.move_file(&PathBuf::from(from), &PathBuf::from(to)).await
        .map_err(|e| ResponseError::GenericError)
}

#[post("/library/<library_id>/files?<path>", data = "<data>")]
pub(crate) async fn upload_library_file(pool: &State<DB>, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str, data: Data<'_>) -> Result<status::NoContent, ResponseError> {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    let mut stream = data.open(MAX_UPLOAD_SIZE);
    // TODO: don't just copy all to memory
    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await.unwrap();

    library.write_file(&PathBuf::from(path), &buf).await
        .map_err(|e| ResponseError::GenericError)?;
    Ok(status::NoContent)
}