use std::net::IpAddr;
use log::debug;
use rocket::{get, post, uri, FromForm, Responder, Route, State};
use rocket::form::{Context, Contextual, Error, Form};
use rocket::form::error::Entity;
use rocket::fs::relative;
use rocket::http::{Header, Status};
use rocket::http::uri::{Origin, Reference, Uri};
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use crate::models::user::{validate_user, try_login_user_form, UserAuthError, UserModel};
use crate::{GlobalMetadata, LoginSessionData, SessionData, DB};
use crate::guards::AuthUser;
use crate::routes::ui;
use crate::routes::ui::user::list_library_files;
use crate::util::{gen_csrf_token, set_csrf, validate_csrf_form, JsonErrorResponse, ResponseError};

pub mod forgot_password;
pub mod login;
pub mod register;

pub mod sso;

#[derive(Responder)]
#[response(status = 302)]
struct HackyRedirectBecauseRocketBug {
    inner: String,
    location: Header<'static>,
}

#[get("/logout")]
pub async fn logout(session: Session<'_, SessionData>, user: AuthUser) -> Redirect {
    session.remove().await.unwrap();
    Redirect::to(uri!(login::page(_, Some(true))))
}


