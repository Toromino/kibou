use database::PooledConnection;

use mastodon_api::{RegistrationForm, StatusForm};
use raito_fe::{renderer, Configuration, LoginForm};
use rocket::http::Cookies;
use rocket::request::LenientForm;
use rocket::response::Redirect;

use rocket_contrib::templates::Template;

#[get("/")]
pub fn index(pooled_connection: PooledConnection, configuration: Configuration) -> Template {
    return renderer::index(&pooled_connection, &configuration);
}

#[get("/about")]
pub fn about(configuration: Configuration) -> Template {
    return renderer::about(&configuration);
}

#[get("/account/<id>", rank = 2)]
pub fn account(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    id: String,
) -> Template {
    return renderer::account_by_local_id(&pooled_connection, &configuration, id);
}

#[get("/account/<id>/follow", rank = 2)]
pub fn account_follow(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    id: i64,
) -> Template {
    return renderer::account_follow(&pooled_connection, &configuration, id, false);
}

#[get("/account/<id>/unfollow", rank = 2)]
pub fn account_unfollow(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    id: i64,
) -> Template {
    return renderer::account_follow(&pooled_connection, &configuration, id, true);
}

#[get("/actors/<handle>", rank = 2)]
pub fn actor(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    handle: String,
) -> Template {
    return renderer::account_by_username(&pooled_connection, &configuration, handle);
}

#[get("/compose?<in_reply_to>", rank = 2)]
pub fn status_draft(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    in_reply_to: Option<i64>,
) -> Template {
    return renderer::compose(&pooled_connection, &configuration, in_reply_to);
}

#[post("/compose", rank = 2, data = "<form>")]
pub fn status_compose(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    form: LenientForm<StatusForm>,
) -> Redirect {
    return renderer::compose_post(&pooled_connection, &configuration, form);
}

#[get("/login")]
pub fn login(pooled_connection: PooledConnection, configuration: Configuration) -> Template {
    return renderer::login(&pooled_connection, &configuration);
}

#[post("/login", data = "<form>")]
pub fn login_post(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    cookies: Cookies,
    form: LenientForm<LoginForm>,
) -> Result<Redirect, Template> {
    return renderer::login_post(&pooled_connection, &configuration, cookies, form);
}

#[get("/timeline/home")]
pub fn home_timeline(
    pooled_connection: PooledConnection,
    configuration: Configuration,
) -> Template {
    return renderer::home_timeline(&pooled_connection, &configuration);
}

#[get("/objects/<id>", rank = 2)]
pub fn object(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    id: String,
) -> Template {
    return renderer::conversation_by_uri(&pooled_connection, &configuration, id);
}

#[post("/register", data = "<form>")]
pub fn register(
    configuration: Configuration,
    cookies: Cookies,
    form: LenientForm<RegistrationForm>,
) -> Result<Redirect, Template> {
    return renderer::register_post(&configuration, cookies, form);
}

#[get("/settings")]
pub fn settings(configuration: Configuration) -> Template {
    return renderer::settings(&configuration);
}

#[get("/status/<id>", rank = 2)]
pub fn view_status(
    pooled_connection: PooledConnection,
    configuration: Configuration,
    id: String,
) -> Template {
    return renderer::conversation(&pooled_connection, &configuration, id);
}

#[get("/timeline/global")]
pub fn global_timeline(
    pooled_connection: PooledConnection,
    configuration: Configuration,
) -> Template {
    return renderer::public_timeline(&pooled_connection, &configuration, false);
}

#[get("/timeline/public")]
pub fn public_timeline(
    pooled_connection: PooledConnection,
    configuration: Configuration,
) -> Template {
    return renderer::public_timeline(&pooled_connection, &configuration, true);
}
