use actor;
use database::PooledConnection;
use lru::LruCache;
use mastodon_api::{
    controller, routes, Account, AuthorizationHeader, HomeTimeline, Notification, PublicTimeline,
    RegistrationForm, Relationship, Status, StatusForm,
};
use oauth;
use raito_fe::{LoginForm, BYPASS_API, MASTODON_API_BASE_URI};
use reqwest::header::{HeaderValue, ACCEPT};
use rocket::request::LenientForm;

pub fn follow(pooled_connection: &PooledConnection, token: &str, id: i64) -> Result<Relationship, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::follow(pooled_connection, token.to_string(), id)
                .to_string(),
        ) {
            Ok(relationship) => Ok(relationship),
            Err(_) => Err(()),
        }
    } else {
        return Err(());
    }
}

pub fn notifications(
    pooled_connection: &PooledConnection,
    token: &str,
) -> Result<Vec<Notification>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::notifications(pooled_connection, token.to_string(), None).to_string(),
        ) {
            Ok(timeline) => Ok(timeline),
            Err(_) => Err(()),
        }
    } else {
        Err(())
    }
}

pub fn unfollow(pooled_connection: &PooledConnection, token: &str, id: i64) -> Result<Relationship, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::unfollow(pooled_connection, token.to_string(), id)
                .to_string(),
        ) {
            Ok(relationship) => Ok(relationship),
            Err(_) => Err(()),
        }
    } else {
        return Err(());
    }
}

pub fn get_account(pooled_connection: &PooledConnection, id: &str) -> Result<Account, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::account(pooled_connection, id.parse::<i64>().unwrap()).to_string(),
        ) {
            Ok(account) => Ok(account),
            Err(_) => Err(()),
        }
    } else {
        match fetch_object(&format!(
            "{base}/api/v1/accounts/{id}",
            base = unsafe { MASTODON_API_BASE_URI },
            id = id
        )) {
            Ok(account) => match serde_json::from_str(&account) {
                Ok(serialized_account) => Ok(serialized_account),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

pub fn get_status(pooled_connection: &PooledConnection, id: String) -> Result<Status, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::status_by_id(pooled_connection, id.parse::<i64>().unwrap()).to_string(),
        ) {
            Ok(status) => Ok(status),
            Err(_) => Err(()),
        }
    } else {
        match fetch_object(&format!(
            "{base}/api/v1/statuses/{id}",
            base = unsafe { MASTODON_API_BASE_URI },
            id = id
        )) {
            Ok(status) => match serde_json::from_str(&status) {
                Ok(serialized_status) => Ok(serialized_status),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

pub fn get_status_context(
    pooled_connection: &PooledConnection,
    id: String,
) -> Result<serde_json::Value, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::context_json_for_id(pooled_connection, id.parse::<i64>().unwrap())
                .to_string(),
        ) {
            Ok(context) => Ok(context),
            Err(_) => Err(()),
        }
    } else {
        match fetch_object(&format!(
            "{base}/api/v1/statuses/{id}/context",
            base = unsafe { MASTODON_API_BASE_URI },
            id = id
        )) {
            Ok(context) => match serde_json::from_str(&context) {
                Ok(status_context) => Ok(status_context),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

pub fn home_timeline(pooled_connection: &PooledConnection, token: &str) -> Result<Vec<Status>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::home_timeline(
                &pooled_connection,
                HomeTimeline {
                    max_id: None,
                    since_id: None,
                    min_id: None,
                    limit: Some(40),
                },
                token.to_string(),
            )
            .to_string(),
        ) {
            Ok(timeline) => Ok(timeline),
            Err(_) => Err(()),
        }
    } else {
        Err(())
    }
}

pub fn get_public_timeline(
    pooled_connection: &PooledConnection,
    local: bool,
) -> Result<Vec<Status>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::public_timeline(
                &pooled_connection,
                PublicTimeline {
                    local: Some(local),
                    only_media: None,
                    max_id: None,
                    since_id: None,
                    min_id: None,
                    limit: Some(40),
                },
            )
            .to_string(),
        ) {
            Ok(timeline) => Ok(timeline),
            Err(_) => Err(()),
        }
    } else {
        match fetch_object(&format!(
            "{base}/api/v1/timelines/public?local={local}&limit=40",
            base = unsafe { MASTODON_API_BASE_URI },
            local = local
        )) {
            Ok(status) => match serde_json::from_str(&status) {
                Ok(serialized_statuses) => Ok(serialized_statuses),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

pub fn get_user_timeline(
    pooled_connection: &PooledConnection,
    id: String,
) -> Result<Vec<Status>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &controller::account_statuses_by_id(
                pooled_connection,
                id.parse::<i64>().unwrap(),
                None,
                None,
                None,
                Some(40),
            )
            .to_string(),
        ) {
            Ok(statuses) => Ok(statuses),
            Err(_) => Err(()),
        }
    } else {
        match fetch_object(&format!(
            "{base}/api/v1/accounts/{id}/statuses?limit=40",
            base = unsafe { MASTODON_API_BASE_URI },
            id = id
        )) {
            Ok(status) => match serde_json::from_str(&status) {
                Ok(serialized_statuses) => Ok(serialized_statuses),
                Err(_) => Err(()),
            },
            Err(_) => Err(()),
        }
    }
}

// This approach is not optimal, as it skips the internal OAuth flow and should definitely be
// reworked. From a security perspective, this approach is safe, as the backend has no reason not
// to trust the internal front-end. On the other hand, this approach does not work if Raito-FE
// is run in standalone.
//
// TODO: Rework
pub fn login(pooled_connection: &PooledConnection, form: LenientForm<LoginForm>) -> Option<String> {
    if unsafe { BYPASS_API } == &true {
        let form = form.into_inner();
        match actor::authorize(pooled_connection, &form.username, form.password) {
            Ok(true) => Some(oauth::token::create(&form.username).access_token),
            Ok(false) => None,
            Err(_) => None,
        }
    } else {
        None
    }
}

pub fn post_status(
    pooled_connection: &PooledConnection,
    form: LenientForm<StatusForm>,
    token: &str,
) {
    if unsafe { BYPASS_API } == &true {
        controller::status_post(pooled_connection, form.into_inner(), token.to_string());
    }
}

// TODO: Rework
// (same as in line 129)
pub fn register(form: LenientForm<RegistrationForm>) -> Option<String> {
    if unsafe { BYPASS_API } == &true {
        let token: Result<oauth::token::Token, serde_json::Error> =
            serde_json::from_value(controller::account_create(&form.into_inner()).into());
        match token {
            Ok(token) => Some(token.access_token),
            Err(_) => None,
        }
    } else {
        None
    }
}

pub fn relationships_by_token(token: &str, ids: Vec<i64>) -> Option<Vec<Relationship>> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_value(controller::relationships(token, ids).into()) {
            Ok(relationships) => Some(relationships),
            Err(_) => None,
        }
    } else {
        // As of now there is no Mastodon-API endpoint for relationships due to limitations of
        // Rocket
        None
    }
}

fn fetch_object(url: &str) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let request = client
        .get(url)
        .header(ACCEPT, HeaderValue::from_static("application/json"))
        .send();

    match request {
        Ok(mut req) => req.text(),
        Err(req) => Err(req),
    }
}
