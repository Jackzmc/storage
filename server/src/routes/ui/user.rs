use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::debug;
use rocket::{get, uri, Response, Route, State};
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Header};
use rocket::http::hyper::body::Buf;
use rocket::response::{status, Redirect, Responder};
use rocket::response::stream::ByteStream;
use rocket::serde::json::{json, Json};
use rocket_dyn_templates::{context, Template};
use serde::Serialize;
use serde_json::Value;
use tokio::sync::Mutex;
use crate::managers::libraries::LibraryManager;
use crate::util::{JsonErrorResponse, ResponseError};

#[get("/")]
pub async fn index(route: &Route) -> Template {
    Template::render("index", context! { route: route.uri.path(), test: "value" })
}

#[get("/library/<library_id>")]
pub async fn redirect_list_library_files(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str)
  -> Result<Redirect, ResponseError>
{
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    Ok(Redirect::to(uri!(list_library_files(library_id, library.model().name, ""))))
}


#[get("/library/<library_id>/<_>/<path..>")]
pub async fn list_library_files(route: &Route, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: PathBuf)
    -> Result<Template, ResponseError>
{
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    let files = library.list_files(&PathBuf::from(&path)).await
        .map_err(|e| ResponseError::InternalServerError(JsonErrorResponse {
            code: "STORAGE_ERROR".to_string(),
            message: e.to_string(),
        }))?;

    // TODO:
    // parent
    let mut parent = path.to_string_lossy();
    if parent == "/" {
        parent = "".into();
    } else if parent != "" {
        parent = format!("{}/", parent).into();
    }
    let mut seg_path = PathBuf::new();
    let segments: Vec<PathSegmentPiece> = path.iter()
        .map(|segment| {
            seg_path = seg_path.join(segment);
            PathSegmentPiece {
                path: seg_path.clone(),
                segment: segment.to_string_lossy().into_owned(),
            }
        })
        .collect();
    debug!("parent={:?}", parent);
    debug!("segments={:?}", segments);

    Ok(Template::render("libraries", context! {
        route: route.uri.path(),
        library: library.model(),
        files: files,
        parent,
        path_segments: segments
    }))
}

#[derive(Debug, Serialize)]
struct PathSegmentPiece {
    pub path: PathBuf,
    pub segment: String
}

#[derive(Responder)]
#[response(status = 200)]
struct FileAttachment {
    content: Vec<u8>,
    // Override the Content-Type declared above.
    content_type: ContentType,
    disposition: Header<'static>,
}

#[get("/file/<library_id>/<path..>")]
pub async fn get_library_file<'a>(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: PathBuf)
    -> Result<FileAttachment, ResponseError>
{
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    match library.read_file(&PathBuf::from(&path)).await
        .map_err(|e| ResponseError::GenericError)?
    {
        None => {
            Err(ResponseError::NotFound(JsonErrorResponse {
                code: "FILE_NOT_FOUND".to_string(),
                message: "Requested file does not exist".to_string()
            }))
        }
        Some(contents) => {
            let file_name = path.file_name().unwrap().to_string_lossy();
            let ext = path.extension().unwrap().to_string_lossy();
            let file_type = ContentType::from_extension(&ext);
            Ok(FileAttachment {
                content: contents,
                content_type: file_type.unwrap_or(ContentType::Binary),
                disposition: Header::new("Content-Disposition", format!("filename=\"{}\"", file_name))
            })
            // Ok(res)
        }
    }

}