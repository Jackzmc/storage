use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;
use log::{debug, error, info, trace, warn};
use rocket::{catch, catchers, launch, routes, uri, Request, Route, State};
use rocket::data::ByteUnit;
use rocket::fs::{relative, FileServer};
use rocket::futures::AsyncWriteExt;
use rocket::http::private::cookie::CookieBuilder;
use rocket::response::Redirect;
use rocket::serde::Serialize;
use rocket_dyn_templates::handlebars::{handlebars_helper, Context, Handlebars, Helper, HelperResult, Output, RenderContext};
use rocket_dyn_templates::{context, Template};
use rocket_session_store::memory::MemoryStore;
use rocket_session_store::SessionStore;
use sqlx::{migrate, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use tokio::sync::Mutex;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use crate::managers::libraries::LibraryManager;
use crate::managers::repos::RepoManager;
use crate::objs::library::Library;
use crate::util::{setup_logger, JsonErrorResponse, ResponseError};
use routes::api;
use crate::consts::{SESSION_COOKIE_NAME, SESSION_LIFETIME_SECONDS};
use crate::models::user::UserModel;
use crate::routes::ui;

mod routes;
mod util;
mod storage;
mod library;
mod user;
mod models;
mod managers;
mod objs;
mod helpers;
mod consts;
mod guards;

pub type DB = Pool<Postgres>;


#[derive(Clone, Debug, Serialize, Default)]
struct SessionData {
    csrf_token: Option<String>,
    login: Option<LoginSessionData>,
}
#[derive(Clone, Debug, Serialize)]
struct LoginSessionData {
    user: UserModel,
    ip_address: IpAddr,
}
#[derive(Clone, Debug, Serialize)]
struct SessionUser {
    id: String,
    name: String,
    email: String
}
#[launch]
async fn rocket() -> _ {
    setup_logger();
    dotenvy::dotenv().ok();

    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(std::env::var("DATABASE_URL").unwrap().as_str())
        .await
        .unwrap();

    migrate!("./migrations")
        .run(&pool)
        .await.unwrap();

    let repo_manager = {
        let mut manager = RepoManager::new(pool.clone());
        manager.fetch_repos().await.unwrap();
        manager
    };
    let libraries_manager = {
        let mut manager = LibraryManager::new(pool.clone(), repo_manager.clone());
        Arc::new(Mutex::new(manager))
    };

    let memory_store: MemoryStore::<SessionData> = MemoryStore::default();
    let store: SessionStore<SessionData> = SessionStore {
        store: Box::new(memory_store),
        name: SESSION_COOKIE_NAME.into(),
        duration: Duration::from_secs(SESSION_LIFETIME_SECONDS),
        // The cookie builder is used to set the cookie's path and other options.
        // Name and value don't matter, they'll be overridden on each request.
        cookie_builder: CookieBuilder::new("", "")
            // Most web apps will want to use "/", but if your app is served from
            // `example.com/myapp/` for example you may want to use "/myapp/" (note the trailing
            // slash which prevents the cookie from being sent for `example.com/myapp2/`).
            .path("/")
    };

    rocket::build()
        .manage(pool)
        .manage(repo_manager)
        .manage(libraries_manager)

        .attach(store.fairing())
        .attach(Template::custom(|engines| {
            let hb = &mut engines.handlebars;

            hb.register_helper("bytes", Box::new(helpers::bytes));
            hb.register_helper("debug", Box::new(helpers::debug));
            hb.register_helper("is-active", Box::new(helpers::is_active));
            hb.register_helper("is-active-exact", Box::new(helpers::is_active));
        }))

        .mount("/static", FileServer::from(relative!("static")))
        .mount("/api/library", routes![
            api::library::move_file, api::library::upload_file, api::library::download_file, api::library::list_files, api::library::get_file, api::library::delete_file,
        ])
        .mount("/auth", routes![
            ui::auth::login, ui::auth::login_handler, ui::auth::register, ui::auth::register_handler,
        ])
        .mount("/", routes![
            ui::help::about,
            ui::user::index, ui::user::redirect_list_library_files, ui::user::list_library_files, ui::user::get_library_file,
            ui::help::test_get
        ])
        .register("/api", catchers![
            not_found_api,
        ])
        .register("/", catchers![
            not_found, not_authorized
        ])
}

#[catch(401)]
pub fn not_authorized(req: &Request) -> Redirect {
    // uri!(ui::auth::login) doesn't work, it redirects to /login instead
    Redirect::to(format!("/auth/login?path={}", req.uri()))
}

#[catch(404)]
fn not_found(req: &Request) -> Template {
    Template::render("errors/404", context! {
        path: req.uri()
    })
}

#[catch(404)]
fn not_found_api(req: &Request) -> ResponseError {
    ResponseError::NotFound(
        JsonErrorResponse {
            code: "ROUTE_NOT_FOUND".to_string(),
            message: "Route not found".to_string(),
        }
    )
}