use rocket::{get, Route};
use rocket::serde::json::Json;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::{Session, SessionResult};
use serde::Serialize;
use crate::models::user::UserModel;
use crate::SessionData;

#[get("/help/about")]
pub fn about(route: &Route) -> Template {
    Template::render("about", context! { route: route.uri.path() })
}


#[get("/test/get")]
pub async fn test_get(session: Session<'_, SessionData>) -> Result<Json<SessionData>, String> {
    session.get().await
        .map_err(|e| e.to_string())?
        .map(|d| Json(d))
        .ok_or_else(|| "Could not find user".to_string())
}