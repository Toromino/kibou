use mastodon_api::account;
use mastodon_api::account::get_json_by_oauth_token;
use mastodon_api::application;
use mastodon_api::application::ApplicationForm;
use mastodon_api::status;
use mastodon_api::status::{post_status, StatusForm};
use mastodon_api::timeline::{
    get_home_timeline_json, get_public_timeline_json, HomeTimeline, PublicTimeline,
};
use mastodon_api::{get_instance_info, parse_authorization_header, AuthorizationHeader};
use oauth::application::Application;
use rocket::request::LenientForm;
use rocket_contrib::json::JsonValue;

#[get("/api/v1/accounts/<id>")]
pub fn account(id: i64) -> JsonValue {
    account::get_json_by_id(id)
}

#[options("/api/v1/accounts/<id>")]
pub fn options_account(id: i64) -> JsonValue {
    account(id)
}

#[get("/api/v1/accounts/verify_credentials")]
pub fn account_verify_credentials(_token: AuthorizationHeader) -> JsonValue {
    get_json_by_oauth_token(parse_authorization_header(&_token.to_string()))
}

#[options("/api/v1/accounts/verify_credentials")]
pub fn options_account_verify_credentials(_token: AuthorizationHeader) -> JsonValue {
    account_verify_credentials(_token)
}

#[post("/api/v1/apps", data = "<form>")]
pub fn application(form: LenientForm<ApplicationForm>) -> JsonValue {
    let form_data: ApplicationForm = form.into_inner();
    application::create_application(Application {
        id: 0,
        client_name: Some(form_data.client_name),
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uris: form_data.redirect_uris,
        scopes: form_data.scopes,
        website: form_data.website,
    })
}

#[get("/api/v1/instance")]
pub fn instance() -> JsonValue {
    get_instance_info()
}

#[options("/api/v1/instance")]
pub fn options_instance() -> JsonValue {
    instance()
}

#[get("/api/v1/statuses/<id>")]
pub fn status(id: i64) -> JsonValue {
    status::get_json_by_id(id)
}

#[options("/api/v1/statuses/<id>")]
pub fn options_status(id: i64) -> JsonValue {
    status(id)
}

#[post("/api/v1/statuses", data = "<form>")]
pub fn status_post(form: LenientForm<StatusForm>, _token: AuthorizationHeader) {
    post_status(
        form.into_inner(),
        parse_authorization_header(&_token.to_string()),
    )
}

#[get("/api/v1/timelines/home?<max_id>&<since_id>&<min_id>&<limit>")]
pub fn home_timeline(
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
    _token: AuthorizationHeader,
) -> JsonValue {
    get_home_timeline_json(
        HomeTimeline {
            max_id,
            since_id,
            min_id,
            limit,
        },
        parse_authorization_header(&_token.to_string()),
    )
}

#[options("/api/v1/timelines/home?<max_id>&<since_id>&<min_id>&<limit>")]
pub fn options_home_timeline(
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
    _token: AuthorizationHeader,
) -> JsonValue {
    home_timeline(max_id, since_id, min_id, limit, _token)
}

#[get("/api/v1/timelines/public?<local>&<only_media>&<max_id>&<since_id>&<min_id>&<limit>")]
pub fn public_timeline(
    local: Option<bool>,
    only_media: Option<bool>,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> JsonValue {
    get_public_timeline_json(PublicTimeline {
        local,
        only_media,
        max_id,
        since_id,
        min_id,
        limit,
    })
}

#[options("/api/v1/timelines/public?<local>&<only_media>&<max_id>&<since_id>&<min_id>&<limit>")]
pub fn options_public_timeline(
    local: Option<bool>,
    only_media: Option<bool>,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> JsonValue {
    public_timeline(local, only_media, max_id, since_id, min_id, limit)
}
