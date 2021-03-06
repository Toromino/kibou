use oauth::authorization::{handle_user_authorization, UserForm};
use oauth::token::{get_token, TokenForm};
use rocket::request::LenientForm;
use rocket::response::Redirect;
use rocket_contrib::json::JsonValue;
use rocket_contrib::templates::Template;
use std::collections::HashMap;

#[get("/oauth/authorize?<styling>")]
pub fn authorize(styling: Option<bool>) -> Template {
    let mut parameters = HashMap::<String, String>::new();
    parameters.insert(String::from("error_context"), String::new());
    parameters.insert(
        String::from("styling"),
        styling.unwrap_or_else(|| true).to_string(),
    );
    Template::render("oauth_authorization", parameters)
}

#[post(
    "/oauth/authorize?<client_id>&<response_type>&<redirect_uri>&<scope>&<state>&<styling>",
    data = "<form>"
)]
pub fn authorize_result(
    client_id: Option<String>,
    response_type: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
    styling: Option<bool>,
    form: LenientForm<UserForm>,
) -> Result<Redirect, Template> {
    handle_user_authorization(
        form.into_inner(),
        client_id,
        response_type,
        redirect_uri,
        state,
        styling,
    )
}

#[post("/oauth/token", data = "<form>")]
pub fn token(form: LenientForm<TokenForm>) -> JsonValue {
    get_token(form.into_inner())
}
