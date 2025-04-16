use std::net::IpAddr;
use rocket::{get, post, FromForm, Route};
use rocket::form::Form;
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use crate::models::user::UserModel;
use crate::{LoginSessionData, SessionData};
use crate::util::{gen_csrf_token, set_csrf, validate_csrf};

#[get("/login")]
pub async fn login(route: &Route, session: Session<'_, SessionData>) -> Template {
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/login", context! {
        route: route.uri.path(),
        csrf_token: csrf_token
    })
}



#[derive(FromForm)]
struct LoginForm<'r> {
    _csrf: &'r str,
    username: &'r str,
    password: &'r str,
    #[field(default = false)]
    remember_me: bool
}
#[post("/login", data = "<form>")]
pub async fn login_handler(route: &Route, ip_addr: IpAddr, form: Form<LoginForm<'_>>, session: Session<'_, SessionData>) -> String {
    if let Ok(true) = validate_csrf(&session, &form._csrf).await {
        if let Ok(sess) = session.get().await.map(|s| s.unwrap_or_default()) {
            session.set(SessionData {
                csrf_token: None,
                login: Some(LoginSessionData {
                    user: UserModel {
                        id: Default::default(),
                        created_at: Default::default(),
                        name: form.username.to_string(),
                    },
                    ip_address: ip_addr,
                }),
            }).await.unwrap();
            return format!("login success")
        }
    }
    format!("login bad. csrf failed!")
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