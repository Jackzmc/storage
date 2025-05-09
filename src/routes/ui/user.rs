use std::cell::OnceCell;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use log::debug;
use rocket::{catch, get, uri, Response, Route, State};
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
use crate::consts::FILE_CONSTANTS;
use crate::guards::{AuthUser};
use crate::managers::libraries::LibraryManager;
use crate::managers::user::UsersState;
use crate::objs::library::ListOptions;
use crate::routes::ui::auth;
use crate::util::{JsonErrorResponse, ResponseError};

#[get("/settings")]
pub async fn user_settings(user: AuthUser, route: &Route) -> Template {
    Template::render("settings", context! { session: user.session, route: route.uri.path() })
}
#[get("/")]
pub async fn index(user: AuthUser, libraries: &State<Arc<Mutex<LibraryManager>>>, route: &Route) -> Template {
    let libraries = libraries.lock().await;
    let list = libraries.list(&user.session.user.id).await.unwrap();
    Template::render("index", context! { session: user.session, libraries: list, route: route.uri.path(), test: "value" })
}

#[get("/library/<library_id>")]
pub async fn redirect_list_library_files(user: AuthUser, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str)
  -> Result<Redirect, ResponseError>
{
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    Ok(Redirect::to(uri!(list_library_files(library_id, library.model().name, "", Some("name"), Some("asc"), Some("list")))))
}

#[get("/library/<library_id>/<_>/<path..>?<sort_key>&<sort_dir>&<display>")]
pub async fn list_library_files(
    user: AuthUser,
    route: &Route,
    libraries: &State<Arc<Mutex<LibraryManager>>>,
    library_id: &str,
    path: PathBuf,
    sort_key: Option<String>,
    sort_dir: Option<String>,
    display: Option<String>,
) -> Result<Template, ResponseError> {
    let options = FileDisplayOptions {
        // TODO: prevent bad values
        // TODO: fix login errror msg -------_____------
        sort_key: validate_option(sort_key, FILE_CONSTANTS.sort_keys, "name"),
        sort_dir: validate_option(sort_dir, &["asc", "desc"], "asc"),
        display: validate_option(display, FILE_CONSTANTS.display_options, "list"),
    };
    let libs = libraries.lock().await;
    let library = libs.get(library_id).await?;
    let list_options = ListOptions {
        sort_field: Some(options.sort_key.clone()),
        sort_descending: Some(options.sort_dir == "desc"),
    };
    let files = library.list_files(&PathBuf::from(&path), list_options).await
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
        session: user.session,
        route: route.uri.path(),
        library: library.model(),
        files: files,
        parent,
        path_segments: segments,
        // TODO: have struct?
        options,
        DATA: FILE_CONSTANTS
    }))
}

/// Checks if option is in list of valid values, if not returns default_value
fn validate_option(option: Option<String>, valid_values: &[&str], default_value: &str) -> String {
    if let Some(option) = option {
        if valid_values.contains(&&*option) {
            return option.to_string()
        }
    }
    default_value.to_string()
}

#[derive(Serialize)]
struct FileDisplayOptions {
    sort_key: String,
    sort_dir: String,
    display: String
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
pub async fn get_library_file<'a>(user: AuthUser, libraries: &State<Arc<Mutex<LibraryManager>>>, library_id: &str, path: PathBuf)
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