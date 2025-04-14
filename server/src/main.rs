use std::sync::Arc;
use log::debug;
use rocket::{catch, launch, routes, Request, State};
use rocket::data::ByteUnit;
use sqlx::{migrate, Pool, Postgres};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::Json;
use tokio::sync::Mutex;
use crate::managers::libraries::LibraryManager;
use crate::managers::repos::RepoManager;
use crate::objs::library::Library;
use crate::util::{setup_logger, JsonErrorResponse, ResponseError};
use routes::api;

mod routes;
mod util;
mod storage;
mod library;
mod user;
mod models;
mod managers;
mod objs;

pub type DB = Pool<Postgres>;

const MAX_UPLOAD_SIZE: ByteUnit = ByteUnit::Mebibyte(100_000);

#[launch]
async fn rocket() -> _ {
    setup_logger();
    dotenvy::dotenv().ok();
    debug!("{}", std::env::var("DATABASE_URL").unwrap().as_str());
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

    rocket::build()
        .manage(pool)
        .manage(repo_manager)
        .manage(libraries_manager)
        .mount("/api", routes![
            api::library::move_file, api::library::upload_file, api::library::download_file, api::library::list_files, api::library::get_file, api::library::delete_file,
        ])
}

#[catch(404)]
fn not_found(req: &Request) -> ResponseError {
    ResponseError::NotFound(
        JsonErrorResponse {
            code: "ROUTE_NOT_FOUND".to_string(),
            message: "Route not found".to_string(),
        }
    )
}