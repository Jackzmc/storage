use rocket::{get, Route};
use rocket::http::Status;
use rocket::response::status;
use rocket_dyn_templates::{context, Template};
use crate::guards::AuthUser;

#[get("/")]
pub async fn index(user: AuthUser, route: &Route) -> Status {
    Status::Forbidden
}