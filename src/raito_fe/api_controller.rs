use actor;
use database;
use mastodon_api::{
    controller, routes, Account, AuthorizationHeader, RegistrationForm, Status, StatusForm,
};
use oauth;
use raito_fe::{LoginForm, BYPASS_API, MASTODON_API_BASE_URI};
use reqwest::header::{HeaderValue, ACCEPT};
use rocket::request::LenientForm;

pub fn get_account(id: String) -> Result<Account, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(&routes::account(id.parse::<i64>().unwrap()).to_string()) {
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

pub fn get_status(id: String) -> Result<Status, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(&routes::status(id.parse::<i64>().unwrap()).to_string()) {
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

pub fn get_status_context(id: String) -> Result<serde_json::Value, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(&routes::status_context(id.parse::<i64>().unwrap()).to_string())
        {
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

pub fn home_timeline(token: &str) -> Result<Vec<Status>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &routes::home_timeline(
                None,
                None,
                None,
                None,
                AuthorizationHeader(token.to_string()),
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

pub fn get_public_timeline(local: bool) -> Result<Vec<Status>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &routes::public_timeline(Some(local), None, None, None, None, None).to_string(),
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

pub fn get_user_timeline(id: String) -> Result<Vec<Status>, ()> {
    if unsafe { BYPASS_API } == &true {
        match serde_json::from_str(
            &routes::account_statuses(
                id.parse::<i64>().unwrap(),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
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
pub fn login(form: LenientForm<LoginForm>) -> Option<String> {
    let db_connection = database::establish_connection();

    if unsafe { BYPASS_API } == &true {
        let form = form.into_inner();
        match actor::authorize(&db_connection, &form.username, form.password) {
            Ok(true) => Some(oauth::token::create(&form.username).access_token),
            Ok(false) => None,
            Err(_) => None,
        }
    } else {
        None
    }
}

pub fn post_status(form: LenientForm<StatusForm>, token: &str) {
    if unsafe { BYPASS_API } == &true {
        routes::status_post(form, AuthorizationHeader(format!("Bearer: {}", token)));
    }
}

// TODO: Rework
// (same as in line 129)
pub fn register(form: LenientForm<RegistrationForm>) -> Option<String> {
    if unsafe { BYPASS_API } == &true {
        match controller::account_create(&form.into_inner()) {
            Some(token) => Some(token.access_token),
            None => None,
        }
    } else {
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
