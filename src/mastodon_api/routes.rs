use mastodon_api::controller;
use mastodon_api::{
    get_instance_info, parse_authorization_header, ApplicationForm, AuthorizationHeader,
    HomeTimeline, PublicTimeline, StatusForm,
};
use oauth::application::Application;
use rocket::request::LenientForm;
use rocket_contrib::json::JsonValue;

#[get("/api/v1/accounts/<id>")]
pub fn account(id: i64) -> JsonValue {
    controller::account_json_by_id(id)
}

#[options("/api/v1/accounts/<id>")]
pub fn options_account(id: i64) -> JsonValue {
    account(id)
}

#[post("/api/v1/accounts/<id>/follow")]
pub fn account_follow(_token: AuthorizationHeader, id: i64) -> JsonValue {
    controller::follow_json(parse_authorization_header(&_token.to_string()), id)
}

#[get("/api/v1/accounts/<id>/statuses?<only_media>&<pinned>&<exclude_replies>&<max_id>&<since_id>&<min_id>&<limit>&<exclude_reblogs>")]
pub fn account_statuses(
    id: i64,
    only_media: Option<bool>,
    pinned: Option<bool>,
    exclude_replies: Option<bool>,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
    exclude_reblogs: Option<bool>,
) -> JsonValue {
    controller::account_statuses_json_by_id(id, max_id, since_id, min_id, limit)
}

#[options("/api/v1/accounts/<id>/statuses?<only_media>&<pinned>&<exclude_replies>&<max_id>&<since_id>&<min_id>&<limit>&<exclude_reblogs>")]
pub fn options_account_statuses(
    id: i64,
    only_media: Option<bool>,
    pinned: Option<bool>,
    exclude_replies: Option<bool>,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
    exclude_reblogs: Option<bool>,
) -> JsonValue {
    account_statuses(
        id,
        only_media,
        pinned,
        exclude_replies,
        max_id,
        since_id,
        min_id,
        limit,
        exclude_reblogs,
    )
}

#[post("/api/v1/accounts/<id>/unfollow")]
pub fn account_unfollow(_token: AuthorizationHeader, id: i64) -> JsonValue {
    controller::unfollow(parse_authorization_header(&_token.to_string()), id)
}

#[get("/api/v1/accounts/verify_credentials")]
pub fn account_verify_credentials(_token: AuthorizationHeader) -> JsonValue {
    controller::account_json_by_oauth_token(parse_authorization_header(&_token.to_string()))
}

#[options("/api/v1/accounts/verify_credentials")]
pub fn options_account_verify_credentials(_token: AuthorizationHeader) -> JsonValue {
    account_verify_credentials(_token)
}

#[post("/api/v1/apps", data = "<form>")]
pub fn application(form: LenientForm<ApplicationForm>) -> JsonValue {
    let form_data: ApplicationForm = form.into_inner();
    controller::application_create(Application {
        id: 0,
        client_name: Some(form_data.client_name),
        client_id: String::new(),
        client_secret: String::new(),
        redirect_uris: form_data.redirect_uris,
        scopes: form_data.scopes,
        website: form_data.website,
    })
}

#[get("/api/v1/custom_emojis")]
pub fn custom_emojis() -> JsonValue {
    return controller::unsupported_endpoint();
}

#[get("/api/v1/filters")]
pub fn filters() -> JsonValue {
    return controller::unsupported_endpoint();
}

#[options("/api/v1/apps", data = "<form>")]
pub fn options_application(form: LenientForm<ApplicationForm>) -> JsonValue {
    application(form)
}

#[get("/api/v1/instance")]
pub fn instance() -> JsonValue {
    get_instance_info()
}

#[get("/api/v1/notifications")]
pub fn notifications() -> JsonValue {
    return controller::unsupported_endpoint();
}

#[options("/api/v1/instance")]
pub fn options_instance() -> JsonValue {
    instance()
}

#[get("/api/v1/statuses/<id>")]
pub fn status(id: i64) -> JsonValue {
    controller::status_json_by_id(id)
}

#[options("/api/v1/statuses/<id>")]
pub fn options_status(id: i64) -> JsonValue {
    status(id)
}

#[get("/api/v1/statuses/<id>/context")]
pub fn status_context(id: i64) -> JsonValue {
    controller::context_json_for_id(id)
}

#[post("/api/v1/statuses", data = "<form>")]
pub fn status_post(form: LenientForm<StatusForm>, _token: AuthorizationHeader) -> JsonValue {
    controller::status_post(
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
    controller::home_timeline_json(
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
    controller::public_timeline_json(PublicTimeline {
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
