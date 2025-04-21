use rocket::{get, post, FromForm, Route, State};
use rocket::form::{Context, Contextual, Form};
use rocket_dyn_templates::{context, Template};
use rocket_session_store::Session;
use crate::{GlobalMetadata, SessionData};
use crate::consts::APP_METADATA;
use crate::util::set_csrf;

#[get("/auth/forgot-password?<return_to>")]
pub async fn page(
    route: &Route,
    session: Session<'_, SessionData>,
    return_to: Option<String>,
) -> Template {
    // TODO: redirect if already logged in
    let csrf_token = set_csrf(&session).await;
    Template::render("auth/forgot-password", context! {
        route: route.uri.path(),
        csrf_token: csrf_token,
        form: &Context::default(),
        return_to,
        meta: APP_METADATA.clone()
    })
}

#[derive(FromForm)]
#[derive(Debug)]
struct ForgotPasswordForm<'r> {
    _csrf: &'r str,
    #[field(validate = len(3..))]
    #[field(validate = contains('@').or_else(msg!("invalid email address")))]
    email: &'r str,
}


#[post("/auth/forgot-password?<return_to>", data = "<form>")]
pub async fn handler(form: Form<Contextual<'_, ForgotPasswordForm<'_>>>, return_to: Option<String>) -> Template {
    todo!()
}