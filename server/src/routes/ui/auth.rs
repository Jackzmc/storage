use rocket::{get, post, Route};
use rocket_dyn_templates::{context, Template};

#[get("/login")]
pub async fn login(route: &Route) -> Template {
    Template::render("auth/login", context! { route: route.uri.path() })

}

#[post("/login")]
pub async fn login_handler(route: &Route) -> Template {
    Template::render("auth/login", context! { route: route.uri.path() })

}

#[get("/register")]
pub async fn register(route: &Route) -> Template {
    Template::render("auth/register", context! { route: route.uri.path() })

}

#[post("/register")]
pub async fn register_handler(route: &Route) -> Template {
    Template::render("auth/register", context! { route: route.uri.path() })

}