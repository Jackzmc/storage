use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use rocket::{get, Response, State};
use rocket::fs::NamedFile;
use rocket::http::{ContentType, Header};
use rocket::http::hyper::body::Buf;
use rocket::response::{status, Responder};
use rocket::response::stream::ByteStream;
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
pub async fn list_library_files(libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: PathBuf)
    -> Result<Template, ResponseError>
{
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
            // TODO: headers?
            let file_name = path.file_name().unwrap().to_string_lossy();
            let ext = path.extension().unwrap().to_string_lossy();
            let file_type = ContentType::from_extension(&ext);
            // let res = Response::build()
            //     .header(file_type.unwrap_or(ContentType::Binary))
            //     .header(Header::new("Content-Disposition", format!("attachment; filename=\"{}\"", file_name)))
            //     .sized_body(contents.len(), Cursor::new(contents))
            //     .finalize();
            Ok(FileAttachment {
                content: contents,
                content_type: file_type.unwrap_or(ContentType::Binary),
                disposition: Header::new("Content-Disposition", format!("filename=\"{}\"", file_name))
            })
            // Ok(res)
        }
    }

}