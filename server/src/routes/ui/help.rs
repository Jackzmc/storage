use rocket::{get, Route};
use rocket_dyn_templates::{context, Template};

#[get("/help/about")]
pub fn about(route: &Route) -> Template {
    Template::render("about", context! { route: route.uri.path() })
}