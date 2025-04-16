use std::net::IpAddr;
use log::debug;
use rocket::{get, post, uri, FromForm, Route, State};
use rocket::form::{Context, Contextual, Error, Form};
use rocket::form::error::Entity;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use crate::models::user::{validate_user, validate_user_form, UserAuthError, UserModel};
use crate::{LoginSessionData, SessionData, DB};
use crate::routes::ui;
use crate::routes::ui::user::list_library_files;
use crate::util::{gen_csrf_token, set_csrf, validate_csrf_form, JsonErrorResponse, ResponseError};

#[get("/login")]
pub async fn login(route: &Route, session: Session<'_, SessionData>) -> Template {
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/login", context! {
        route: route.uri.path(),
        csrf_token: csrf_token,
        form: &Context::default(),
    })
}

#[derive(FromForm)]
#[derive(Debug)]
struct LoginForm<'r> {
    _csrf: &'r str,
    #[field(validate = len(1..))]
    username: &'r str,
    #[field(validate = len(1..))]
    password: &'r str,
    #[field(default = false)]
    remember_me: bool
}



#[post("/login", data = "<form>")]
pub async fn login_handler(
    pool: &State<DB>,
    route: &Route,
    ip_addr: IpAddr,
    session: Session<'_, SessionData>,
    mut form: Form<Contextual<'_, LoginForm<'_>>>,
) -> Result<Redirect, Template> {
    validate_csrf_form(&mut form.context, &session).await;
    let user = validate_user_form(&mut form.context, &pool).await;
    if form.context.status() == Status::Ok {
        if let Some(submission) = &form.value {
            session.set(SessionData {
                csrf_token: None,
                login: Some(LoginSessionData {
                    user: user.expect("failed to acquire user but no errors"), // if validate_user_form returned None, form had errors, this shouldnt run,
                    ip_address: ip_addr,
                }),
            }).await.unwrap();

            return Ok(Redirect::to(uri!(ui::user::index())))
        }
    }

    let csrf_token = set_csrf(&session).await;
    let ctx = context! {
        csrf_token,
        form: &form.context
    };
    Err(Template::render("auth/login", &ctx))
}

#[get("/register")]
pub async fn register(route: &Route, session: Session<'_, SessionData>) -> Template {
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/register", context! {
        route: route.uri.path(),
        csrf_token: csrf_token
    })
}

#[post("/register")]
pub async fn register_handler(route: &Route, session: Session<'_, SessionData>) -> Template {
    Template::render("auth/register", context! { route: route.uri.path() })

}