use rocket::{get, post, Route, State};
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use crate::{GlobalMetadata, SessionData};
use crate::consts::APP_METADATA;
use crate::util::set_csrf;

#[get("/auth/register")]
pub async fn page(route: &Route, session: Session<'_, SessionData>) -> Template {
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/register", context! {
        route: route.uri.path(),
        csrf_token: csrf_token,
        meta: APP_METADATA.clone()
    })
}

#[post("/auth/register")]
pub async fn handler(route: &Route, session: Session<'_, SessionData>) -> Template {
    Template::render("auth/register", context! { route: route.uri.path() })

}