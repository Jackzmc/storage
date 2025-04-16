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
use crate::models::user::{validate_user, validate_user_form, UserAuthError, UserModel};
use crate::{LoginSessionData, SessionData, DB};
use crate::guards::AuthUser;
use crate::routes::ui;
use crate::routes::ui::user::list_library_files;
use crate::util::{gen_csrf_token, set_csrf, validate_csrf_form, JsonErrorResponse, ResponseError};

#[get("/logout")]
pub async fn logout(session: Session<'_, SessionData>, user: AuthUser) -> Redirect {
    session.remove().await.unwrap();
    Redirect::to(uri!("/auth", login(_, Some(true))))
}

#[get("/login?<return_to>&<logged_out>")]
pub async fn login(route: &Route, session: Session<'_, SessionData>, return_to: Option<String>, logged_out: Option<bool>) -> Template {
    // TODO: redirect if already logged in
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/login", context! {
        route: route.uri.path(),
        csrf_token: csrf_token,
        form: &Context::default(),
        return_to,
        logged_out
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


#[derive(Responder)]
#[response(status = 302)]
struct HackyRedirectBecauseRocketBug {
    inner: String,
    location: Header<'static>,
}

#[post("/login?<return_to>", data = "<form>")]
pub async fn login_handler(
    pool: &State<DB>,
    route: &Route,
    ip_addr: IpAddr,
    session: Session<'_, SessionData>,
    mut form: Form<Contextual<'_, LoginForm<'_>>>,
    return_to: Option<String>,
) -> Result<HackyRedirectBecauseRocketBug, Template> {
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
            debug!("returning user to {:?}", return_to);
            let return_to_path = return_to.unwrap_or("/".to_string());
            // Rocket redirect fails when `Redirect::to("/path/ has spaces")` has spaces, so manually do location... works better
            return Ok(HackyRedirectBecauseRocketBug {
                inner: "Login successful, redirecting...".to_string(),
                location: Header::new("Location", return_to_path),
            })
            // let return_to_uri = Uri::parse::<Origin>(&return_to_path).unwrap_or(Uri::parse::<Origin>("/").unwrap());
            // return Ok(Redirect::found(return_to_uri))
        }
    }

    let csrf_token = set_csrf(&session).await;
    let ctx = context! {
        csrf_token,
        form: &form.context,
        return_to
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