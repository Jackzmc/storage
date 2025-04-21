use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use log::{debug, error, info, trace, warn};
use rocket::{catch, catchers, launch, routes, uri, Request, Route, State};
use rocket::data::ByteUnit;
use rocket::fs::{relative, FileServer};
use rocket::futures::AsyncWriteExt;
use rocket::http::private::cookie::CookieBuilder;
use rocket::http::uri::Uri;
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
use crate::util::{setup_db, setup_logger, setup_session_store, JsonErrorResponse, ResponseError};
use routes::api;
use crate::config::{get_settings, AppConfig};
use crate::consts::{init_statics, SESSION_COOKIE_NAME, SESSION_LIFETIME_SECONDS};
use crate::managers::sso::{SSOState, SSO};
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
mod config;

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
#[derive(Serialize, Clone)]
pub struct GlobalMetadata {
    app_name: String,
    app_version: String,
    repo_url: String,
}
#[launch]
async fn rocket() -> _ {
    setup_logger();
    dotenvy::dotenv().ok();
    init_statics();

    trace!("trace");
    debug!("debug");
    info!("info");
    warn!("warn");
    error!("error");

    let settings: AppConfig = get_settings();
    info!("Auth | Registration={} Login={} | OIDC={} CreateAccount={}",
        if settings.auth.disable_registration { "N" } else { "Y" },
        settings.auth.oidc.as_ref().map(|oidc| if oidc.disable_normal_login { "N" } else { "Y" } ).unwrap_or("Y"),
        settings.auth.oidc.as_ref().map(|oidc| if oidc.enabled { "Y" } else { "N" } ).unwrap_or("N"),
        settings.auth.oidc.as_ref().map(|oidc| if oidc.create_account { "Y" } else { "N" }).unwrap_or("-"),
    );
    let listen_ip: IpAddr = settings.general.listen_ip.as_ref()
        .map(|s| s.to_string())
        .unwrap_or_else(||"0.0.0.0".to_string())
        .parse().expect("bad listen ip");
    let listen_addr = SocketAddr::new(listen_ip, settings.general.listen_port.unwrap_or(8080));
    info!("Listening on {} | Public URL: {}", listen_addr, settings.general.public_url);
    if let Some(ref smtp) = settings.smtp {
        if smtp.enabled {
            info!("SMTP Enabled");
        }
    }
    let pool = setup_db().await;

    let repo_manager = {
        let mut manager = RepoManager::new(pool.clone());
        manager.fetch_repos().await.unwrap();
        manager
    };
    let libraries_manager = {
        let mut manager = LibraryManager::new(pool.clone(), repo_manager.clone());
        Arc::new(Mutex::new(manager))
    };

    // TODO: move to own func
    let store = setup_session_store();
    let sso: SSOState = {
        if settings.auth.oidc.is_some() { Some(Arc::new(Mutex::new(SSO::create(&settings).await)) ) } else { None }
    };

    let figment = rocket::Config::figment()
        .merge(("port", listen_addr.port()))
        .merge(("address", listen_addr.ip()));

    rocket::custom(figment)
        .manage(pool)
        .manage(repo_manager)
        .manage(libraries_manager)
        .manage(settings)
        .manage(sso)

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
        .mount("/", routes![
            ui::auth::logout,
            ui::auth::login::page, ui::auth::login::handler, ui::auth::register::page, ui::auth::register::handler,
            ui::auth::sso::page, ui::auth::sso::callback,
            ui::auth::forgot_password::page, ui::auth::forgot_password::handler,
        ])
        .mount("/", routes![
            ui::user::user_settings, ui::user::index, ui::user::redirect_list_library_files, ui::user::list_library_files, ui::user::get_library_file,
        ])
        .mount("/", routes![
            ui::help::about,
            ui::help::test_get
        ])
        .mount("/admin", routes![
            ui::admin::index
        ])
        .register("/api", catchers![
            not_found_api,
        ])
        .register("/", catchers![
            not_found, not_authorized, forbidden
        ])
}

#[catch(401)]
pub fn not_authorized(req: &Request) -> Redirect {
    // TODO: do uri!()
    Redirect::to(format!("/auth/login?return_to={}", req.uri().path().percent_encode()))
}

#[catch(403)]
pub fn forbidden(req: &Request) -> Template {
   Template::render("errors/403", context! {

   })
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