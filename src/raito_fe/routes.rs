use mastodon_api::{RegistrationForm, StatusForm};
use raito_fe::{renderer, Authentication, LocalConfiguration, LoginForm};
use rocket::http::Cookies;
use rocket::request::LenientForm;
use rocket_contrib::templates::Template;

#[get("/")]
pub fn index(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    return renderer::index(configuration, authentication);
}

#[get("/about")]
pub fn about(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    return renderer::about(configuration, authentication);
}

#[get("/account/<id>", rank = 2)]
pub fn account(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> Template {
    return renderer::account_by_local_id(configuration, authentication, id);
}

#[get("/account/<id>/follow", rank = 2)]
pub fn account_follow(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: i64,
) -> Template {
    return renderer::account_follow(configuration, authentication, id, false);
}

#[get("/account/<id>/unfollow", rank = 2)]
pub fn account_unfollow(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: i64,
) -> Template {
    return renderer::account_follow(configuration, authentication, id, true);
}

#[get("/actors/<handle>", rank = 2)]
pub fn actor(
    configuration: LocalConfiguration,
    authentication: Authentication,
    handle: String,
) -> Template {
    return renderer::account_by_username(configuration, authentication, handle);
}

#[get("/compose?<in_reply_to>", rank = 2)]
pub fn status_draft(
    configuration: LocalConfiguration,
    authentication: Authentication,
    in_reply_to: Option<i64>,
) -> Template {
    return renderer::compose(configuration, authentication, in_reply_to);
}

#[post("/compose", rank = 2, data = "<form>")]
pub fn status_compose(
    configuration: LocalConfiguration,
    authentication: Authentication,
    form: LenientForm<StatusForm>,
) -> Template {
    return renderer::compose_post(configuration, authentication, form);
}

#[get("/login")]
pub fn login(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    return renderer::login(configuration, authentication);
}

#[post("/login", data = "<form>")]
pub fn login_post(
    configuration: LocalConfiguration,
    authentication: Authentication,
    cookies: Cookies,
    form: LenientForm<LoginForm>,
) -> Template {
    return renderer::login_post(configuration, authentication, cookies, form);
}

#[get("/timeline/home")]
pub fn home_timeline(
    configuration: LocalConfiguration,
    authentication: Authentication,
) -> Template {
    return renderer::home_timeline(configuration, authentication);
}

#[get("/objects/<id>", rank = 2)]
pub fn object(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> Template {
    return renderer::conversation_by_uri(configuration, authentication, id);
}

#[post("/register", data = "<form>")]
pub fn register(
    configuration: LocalConfiguration,
    authentication: Authentication,
    cookies: Cookies,
    form: LenientForm<RegistrationForm>,
) -> Template {
    return renderer::register_post(configuration, authentication, cookies, form);
}

#[get("/settings")]
pub fn settings(configuration: LocalConfiguration, authentication: Authentication) -> Template {
    return renderer::settings(configuration, authentication);
}

#[get("/status/<id>", rank = 2)]
pub fn view_status(
    configuration: LocalConfiguration,
    authentication: Authentication,
    id: String,
) -> Template {
    return renderer::conversation(configuration, authentication, id);
}

#[get("/timeline/global")]
pub fn global_timeline(
    configuration: LocalConfiguration,
    authentication: Authentication,
) -> Template {
    return renderer::public_timeline(configuration, authentication, false);
}

#[get("/timeline/public")]
pub fn public_timeline(
    configuration: LocalConfiguration,
    authentication: Authentication,
) -> Template {
    return renderer::public_timeline(configuration, authentication, true);
}
