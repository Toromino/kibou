use database::PooledConnection;

use mastodon_api::controller;
use mastodon_api::{
    parse_authorization_header, ApplicationForm, AuthorizationHeader, HomeTimeline, PublicTimeline,
    StatusForm,
};
use oauth::application::Application;
use rocket::request::LenientForm;

use rocket_contrib::json::JsonValue;

#[get("/api/v1/accounts/<id>")]
pub fn account(pooled_connection: PooledConnection, id: i64) -> JsonValue {
    return controller::account(&pooled_connection, id);
}

#[options("/api/v1/accounts/<id>")]
pub fn options_account(pooled_connection: PooledConnection, id: i64) -> JsonValue {
    return account(pooled_connection, id);
}

#[post("/api/v1/accounts/<id>/follow")]
pub fn account_follow(_token: AuthorizationHeader, id: i64) -> JsonValue {
    return controller::follow(parse_authorization_header(&_token.to_string()), id);
}

#[get("/api/v1/accounts/<id>/statuses?<only_media>&<pinned>&<exclude_replies>&<max_id>&<since_id>&<min_id>&<limit>&<exclude_reblogs>")]
pub fn account_statuses(
    pooled_connection: PooledConnection,
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
    return controller::account_statuses_by_id(
        &pooled_connection,
        id,
        max_id,
        since_id,
        min_id,
        limit,
    );
}

#[options("/api/v1/accounts/<id>/statuses?<only_media>&<pinned>&<exclude_replies>&<max_id>&<since_id>&<min_id>&<limit>&<exclude_reblogs>")]
pub fn options_account_statuses(
    pooled_connection: PooledConnection,
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
    return account_statuses(
        pooled_connection,
        id,
        only_media,
        pinned,
        exclude_replies,
        max_id,
        since_id,
        min_id,
        limit,
        exclude_reblogs,
    );
}

#[post("/api/v1/accounts/<id>/unfollow")]
pub fn account_unfollow(_token: AuthorizationHeader, id: i64) -> JsonValue {
    return controller::unfollow(parse_authorization_header(&_token.to_string()), id);
}

#[get("/api/v1/accounts/verify_credentials")]
pub fn account_verify_credentials(
    pooled_connection: PooledConnection,
    _token: AuthorizationHeader,
) -> JsonValue {
    return controller::account_by_oauth_token(
        &pooled_connection,
        parse_authorization_header(&_token.to_string()),
    );
}

#[options("/api/v1/accounts/verify_credentials")]
pub fn options_account_verify_credentials(
    pooled_connection: PooledConnection,
    _token: AuthorizationHeader,
) -> JsonValue {
    return account_verify_credentials(pooled_connection, _token);
}

#[post("/api/v1/apps", data = "<form>")]
pub fn application(
    pooled_connection: PooledConnection,
    form: LenientForm<ApplicationForm>,
) -> JsonValue {
    let form_data: ApplicationForm = form.into_inner();
    return controller::application_create(
        &pooled_connection,
        Application {
            id: 0,
            client_name: Some(form_data.client_name),
            client_id: String::new(),
            client_secret: String::new(),
            redirect_uris: form_data.redirect_uris,
            scopes: form_data.scopes,
            website: form_data.website,
        },
    );
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
pub fn options_application(
    pooled_connection: PooledConnection,
    form: LenientForm<ApplicationForm>,
) -> JsonValue {
    return application(pooled_connection, form);
}

#[get("/api/v1/instance")]
pub fn instance() -> JsonValue {
    return controller::instance_info();
}

#[get("/api/v1/notifications?<limit>")]
pub fn notifications(
    pooled_connection: PooledConnection,
    _token: AuthorizationHeader,
    limit: Option<i64>,
) -> JsonValue {
    return controller::notifications(
        &pooled_connection,
        parse_authorization_header(&_token.to_string()),
        limit,
    );
}

#[options("/api/v1/instance")]
pub fn options_instance() -> JsonValue {
    return controller::instance_info();
}

#[get("/api/v1/statuses/<id>")]
pub fn status(pooled_connection: PooledConnection, id: i64) -> JsonValue {
    return controller::status_by_id(&pooled_connection, id);
}

#[options("/api/v1/statuses/<id>")]
pub fn options_status(pooled_connection: PooledConnection, id: i64) -> JsonValue {
    return status(pooled_connection, id);
}

#[get("/api/v1/statuses/<id>/context")]
pub fn status_context(pooled_connection: PooledConnection, id: i64) -> JsonValue {
    return controller::context_json_for_id(&pooled_connection, id);
}

#[post("/api/v1/statuses/<id>/favourite")]
pub fn status_favourite(
    pooled_connection: PooledConnection,
    _token: AuthorizationHeader,
    id: i64,
) -> JsonValue {
    return controller::favourite(
        &pooled_connection,
        parse_authorization_header(&_token.to_string()),
        id,
    );
}

#[post("/api/v1/statuses", data = "<form>")]
pub fn status_post(
    pooled_connection: PooledConnection,
    form: LenientForm<StatusForm>,
    _token: AuthorizationHeader,
) -> JsonValue {
    return controller::status_post(
        &pooled_connection,
        form.into_inner(),
        parse_authorization_header(&_token.to_string()),
    );
}

#[post("/api/v1/statuses/<id>/reblog")]
pub fn status_reblog(
    pooled_connection: PooledConnection,
    _token: AuthorizationHeader,
    id: i64,
) -> JsonValue {
    return controller::reblog(
        &pooled_connection,
        parse_authorization_header(&_token.to_string()),
        id,
    );
}

#[get("/api/v1/timelines/home?<max_id>&<since_id>&<min_id>&<limit>")]
pub fn home_timeline(
    pooled_connection: PooledConnection,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
    _token: AuthorizationHeader,
) -> JsonValue {
    return controller::home_timeline(
        &pooled_connection,
        HomeTimeline {
            max_id,
            since_id,
            min_id,
            limit,
        },
        parse_authorization_header(&_token.to_string()),
    );
}

#[options("/api/v1/timelines/home?<max_id>&<since_id>&<min_id>&<limit>")]
pub fn options_home_timeline(
    pooled_connection: PooledConnection,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
    _token: AuthorizationHeader,
) -> JsonValue {
    return home_timeline(pooled_connection, max_id, since_id, min_id, limit, _token);
}

#[get("/api/v1/timelines/public?<local>&<only_media>&<max_id>&<since_id>&<min_id>&<limit>")]
pub fn public_timeline(
    pooled_connection: PooledConnection,
    local: Option<bool>,
    only_media: Option<bool>,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> JsonValue {
    return controller::public_timeline(
        &pooled_connection,
        PublicTimeline {
            local,
            only_media,
            max_id,
            since_id,
            min_id,
            limit,
        },
    );
}

#[options("/api/v1/timelines/public?<local>&<only_media>&<max_id>&<since_id>&<min_id>&<limit>")]
pub fn options_public_timeline(
    pooled_connection: PooledConnection,
    local: Option<bool>,
    only_media: Option<bool>,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> JsonValue {
    return public_timeline(
        pooled_connection,
        local,
        only_media,
        max_id,
        since_id,
        min_id,
        limit,
    );
}
