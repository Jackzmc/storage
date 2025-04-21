use std::net::IpAddr;
use log::{debug, trace};
use rocket::{get, post, FromForm, Responder, Route, State};
use rocket::form::{Context, Contextual, Form};
use rocket::http::{Header, Status};
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use crate::{GlobalMetadata, LoginSessionData, SessionData, DB};
use crate::consts::{APP_METADATA, DISABLE_LOGIN_CHECK};
use crate::models::user::validate_user_form;
use crate::routes::ui::auth::HackyRedirectBecauseRocketBug;
use crate::util::{set_csrf, validate_csrf_form};

#[get("/auth/login?<return_to>&<logged_out>")]
pub async fn page(
    route: &Route,
    session: Session<'_, SessionData>,
    return_to: Option<String>,
    logged_out: Option<bool>
) -> Template {
    // TODO: redirect if already logged in
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/login", context! {
        route: route.uri.path(),
        csrf_token: csrf_token,
        form: &Context::default(),
        return_to,
        logged_out,
        meta: APP_METADATA.clone()
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


#[post("/auth/login?<return_to>", data = "<form>")]
pub async fn handler(
    pool: &State<DB>,
    route: &Route,
    ip_addr: IpAddr,
    session: Session<'_, SessionData>,
    mut form: Form<Contextual<'_, LoginForm<'_>>>,
    return_to: Option<String>,
) -> Result<HackyRedirectBecauseRocketBug, Template> {
    trace!("handler");
    if !*DISABLE_LOGIN_CHECK {
        validate_csrf_form(&mut form.context, &session).await;
    }
    let user = validate_user_form(&mut form.context, &pool).await;
    trace!("check form");
    if form.context.status() == Status::Ok {
        if let Some(submission) = &form.value {
            session.set(SessionData {
                csrf_token: None,
                login: Some(LoginSessionData {
                    user: user.expect("failed to acquire user but no errors"), // if validate_user_form returned None, form had errors, this shouldnt run,
                    ip_address: ip_addr,
                }),
            }).await.unwrap();
            let mut return_to_path = return_to.unwrap_or("/".to_string());
            if return_to_path == "" { return_to_path.push_str("/"); }
            debug!("returning user to {:?}", return_to_path);

            // Rocket redirect fails when `Redirect::to("/path/ has spaces")` has spaces, so manually do location... works better
            return Ok(HackyRedirectBecauseRocketBug {
                inner: "Login successful, redirecting...".to_string(),
                location: Header::new("Location", return_to_path),
            })
        }
        trace!("submission failed");
    }
    trace!("form failed");

    let csrf_token = set_csrf(&session).await;
    let ctx = context! {
        route: route.uri.path(),
        csrf_token: csrf_token,
        form: &form.context,
        return_to,
        meta: APP_METADATA.clone()
    };
    Err(Template::render("auth/login", &ctx))
}