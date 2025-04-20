use std::path::PathBuf;
use std::sync::Arc;
use log::debug;
use rocket::{delete, get, post, Data, Route, State};
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::response::status;
use rocket::serde::json::Json;
use rocket::serde::Serialize;
use sqlx::{query, Postgres};
use sqlx::types::{Uuid};
use tokio::io::AsyncReadExt;
use tokio::sync::Mutex;
use crate::{library, models, DB};
use crate::consts::MAX_UPLOAD_SIZE;
use crate::managers::libraries::LibraryManager;
use crate::managers::repos::RepoManager;
use crate::models::library::{LibraryModel, LibraryWithRepoModel};
use crate::models::user;
use crate::objs::library::ListOptions;
use crate::storage::{FileEntry, FileType};
use crate::util::{JsonErrorResponse, ResponseError};
#[get("/<library_id>")]
pub(crate) async fn get_file(pool: &State<DB>, library_id: &str) -> Result<Option<Json<LibraryWithRepoModel>>, ResponseError> {
    let library = models::library::get_library_with_repo(pool, library_id).await
        .map_err(|e| ResponseError::GenericError)?;
    Ok(library.map(|lib| Json(lib)))
}

#[get("/<library_id>/files?<path>")]
pub(crate) async fn list_files(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str) -> Result<Json<Vec<FileEntry>>, ResponseError> {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    library.list_files(&PathBuf::from(path), ListOptions::default()).await
        .map(|files| Json(files))
        .map_err(|e| ResponseError::InternalServerError(JsonErrorResponse {
            code: "STORAGE_ERROR".to_string(),
            message: e.to_string(),
        }))
}


#[post("/<library_id>/touch?<path>&<file_type>")]
pub(crate) async fn touch_files(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str, file_type: FileType) -> Result<(), ResponseError> {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    library.touch_file(&PathBuf::from(path), file_type).await
        .map_err(|e| ResponseError::InternalServerError(JsonErrorResponse {
            code: "STORAGE_ERROR".to_string(),
            message: e.to_string(),
        }))
}

#[get("/<library_id>/files/download?<path>")]
pub(crate) async fn download_file(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str) -> Result<Vec<u8>, ResponseError>   {
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

#[post("/<library_id>/files/move?<from>&<to>")]
pub(crate) async fn move_file(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, from: &str, to: &str) -> Result<(), ResponseError>   {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    library.move_file(&PathBuf::from(from), &PathBuf::from(to)).await
        .map_err(|e| ResponseError::GenericError)
}

#[post("/<library_id>/files?<path>", data = "<data>")]
pub(crate) async fn upload_file(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str, data: Data<'_>) -> Result<status::NoContent, ResponseError> {
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

#[delete("/<library_id>/files/move?<path>")]
pub(crate) async fn delete_file(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: &str) -> Result<(), ResponseError>   {
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    library.delete_file(&PathBuf::from(path)).await
        .map_err(|e| ResponseError::GenericError)
}

