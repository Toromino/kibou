use activity;
use activity::{
    get_activities_by_id, get_ap_object_by_id, get_ap_object_replies_by_id,
    type_exists_for_object_id,
};
use activitypub;
use actor;
use actor::get_actor_by_id;
use actor::get_actor_by_uri;
use chrono;
use chrono::Utc;
use core::borrow::Borrow;
use database;
use database::PooledConnection;
use diesel::PgConnection;
use env;
use kibou_api;
use lru::LruCache;
use mastodon_api::routes::status;
use mastodon_api::{
    Account, Attachment, HomeTimeline, Instance, Notification, PublicTimeline, RegistrationForm,
    Relationship, Source, Status, StatusForm, MASTODON_API_ACCOUNT_CACHE,
    MASTODON_API_NOTIFICATION_CACHE, MASTODON_API_STATUS_CACHE,
};
use notification::notifications_for_actor;
use oauth;
use oauth::application::Application as OAuthApplication;
use oauth::token::{verify_token, Token};
use regex::Regex;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use timeline;
use timeline::{home_timeline as get_home_timeline, public_timeline as get_public_timeline};

pub fn account(pooled_connection: &PooledConnection, id: i64) -> JsonValue {
    match actor::get_actor_by_id(pooled_connection, &id) {
        Ok(actor) => json!(Account::from_actor(pooled_connection, actor, false)),
        Err(_) => json!({"error": "User not found."}),
    }
}

pub fn account_by_oauth_token(pooled_connection: &PooledConnection, token: String) -> JsonValue {
    match verify_token(pooled_connection, token) {
        Ok(token) => {
            match actor::get_local_actor_by_preferred_username(pooled_connection, &token.actor) {
                Ok(actor) => json!(Account::from_actor(pooled_connection, actor, true)),
                Err(_) => json!({"error": "No user is associated to this token!"}),
            }
        }
        Err(_) => json!({"error": "Token invalid!"}),
    }
}

pub fn account_create(form: &RegistrationForm) -> JsonValue {
    let email_regex = Regex::new(r"^[a-zA-Z0-9_.+-]+@[a-zA-Z0-9-]+\.[a-zA-Z0-9-.]+$").unwrap();
    let username_regex = Regex::new(r"^[A-Za-z0-9_]{1,32}$").unwrap();

    if username_regex.is_match(&form.username) && email_regex.is_match(&form.email) {
        let database = database::establish_connection();
        let mut new_actor = actor::Actor {
            id: 0,
            email: Some(form.email.to_string()),
            password: Some(form.password.to_string()),
            actor_uri: format!(
                "{base_scheme}://{base_domain}/actors/{username}",
                base_scheme = env::get_value(String::from("endpoint.base_scheme")),
                base_domain = env::get_value(String::from("endpoint.base_domain")),
                username = form.username
            ),
            username: Some(form.username.to_string()),
            preferred_username: form.username.to_string(),
            summary: None,
            followers: serde_json::json!({"activitypub": []}),
            inbox: None,
            icon: None,
            local: true,
            keys: serde_json::json!({}),
            created: Utc::now().naive_utc(),
            modified: Utc::now().naive_utc(),
        };

        actor::create_actor(&database, &mut new_actor);

        match actor::get_local_actor_by_preferred_username(&database, &form.username) {
            Ok(_actor) => json!(oauth::token::create(&form.username)),
            Err(_) => json!({"error": "Account could not be created"}),
        }
    } else {
        return json!({"error": "Username or E-mail contains unsupported characters"});
    }
}

pub fn account_statuses_by_id(
    pooled_connection: &PooledConnection,
    id: i64,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> JsonValue {
    match actor::get_actor_by_id(pooled_connection, &id) {
        Ok(actor) => {
            match timeline::user_timeline(pooled_connection, actor, max_id, since_id, min_id, limit)
            {
                Ok(statuses) => cached_statuses(pooled_connection, statuses),
                Err(_) => json!({"error": "Error generating user timeline."}),
            }
        }
        Err(_) => json!({"error": "User not found."}),
    }
}

pub fn application_create(
    pooled_connection: &PooledConnection,
    application: OAuthApplication,
) -> JsonValue {
    let oauth_app: OAuthApplication = oauth::application::create(pooled_connection, application);
    rocket_contrib::json!({
        "name": oauth_app.client_name.unwrap_or_default(),
        "website": oauth_app.website,
        "client_id": oauth_app.client_id,
        "client_secret": oauth_app.client_secret,
        "redirect_uri": oauth_app.redirect_uris,
        "id": oauth_app.id
    })
}

pub fn cached_account(pooled_connection: &PooledConnection, uri: &str) -> JsonValue {
    let mut account_cache = MASTODON_API_ACCOUNT_CACHE.lock().unwrap();

    if account_cache.contains(&uri.to_string()) {
        return json!(account_cache.get(&uri.to_string()));
    } else {
        match actor::get_actor_by_uri(pooled_connection, uri) {
            Ok(actor) => {
                let result =
                    serde_json::json!(Account::from_actor(pooled_connection, actor, false));
                account_cache.put(uri.to_string(), result.clone());
                return json!(result);
            }
            Err(_) => json!({ "error": format!("Account not found: {}", uri) }),
        }
    }
}

pub fn cached_notifications(pooled_connection: &PooledConnection, ids: Vec<i64>) -> JsonValue {
    let mut notification_cache = MASTODON_API_NOTIFICATION_CACHE.lock().unwrap();
    let mut notifications: Vec<Notification> = Vec::new();
    let mut uncached_notifications: Vec<i64> = Vec::new();
    for id in ids {
        if notification_cache.contains(&id) {
            notifications.push(
                serde_json::from_value(notification_cache.get(&id).unwrap().clone()).unwrap(),
            );
        } else {
            uncached_notifications.push(id);
        }
    }

    match get_activities_by_id(pooled_connection, uncached_notifications) {
        Ok(activities) => {
            for activity in activities {
                match Notification::try_from(activity) {
                    Ok(notification) => {
                        notification_cache.put(
                            notification.id.parse::<i64>().unwrap(),
                            serde_json::json!(notification),
                        );
                        notifications.push(notification);
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    notifications.sort_by(|a, b| b.id.cmp(&a.id));
    return json!(notifications);
}

pub fn cached_statuses(pooled_connection: &PooledConnection, ids: Vec<i64>) -> JsonValue {
    let mut status_cache = MASTODON_API_STATUS_CACHE.lock().unwrap();
    let mut statuses: Vec<Status> = Vec::new();
    let mut uncached_statuses: Vec<i64> = Vec::new();
    for id in ids {
        if status_cache.contains(&id) {
            statuses.push(serde_json::from_value(status_cache.get(&id).unwrap().clone()).unwrap());
        } else {
            uncached_statuses.push(id);
        }
    }

    match get_activities_by_id(pooled_connection, uncached_statuses) {
        Ok(activities) => {
            for activity in activities {
                match Status::try_from(activity) {
                    Ok(status) => {
                        status_cache
                            .put(status.id.parse::<i64>().unwrap(), serde_json::json!(status));
                        statuses.push(status);
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    statuses.sort_by(|a, b| b.id.cmp(&a.id));
    return json!(statuses);
}

pub fn context_json_for_id(pooled_connection: &PooledConnection, id: i64) -> JsonValue {
    match activity::get_activity_by_id(&pooled_connection, id) {
        Ok(_activity) => {
            let mut ancestors = status_parents_for_id(pooled_connection, id, true);
            let mut descendants = status_children_for_id(pooled_connection, id, true);
            ancestors.sort_by(|status_a, status_b| {
                chrono::DateTime::parse_from_rfc3339(&status_a.created_at)
                    .unwrap_or_else(|_| {
                        chrono::DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap()
                    })
                    .timestamp()
                    .cmp(
                        &chrono::DateTime::parse_from_rfc3339(&status_b.created_at)
                            .unwrap_or_else(|_| {
                                chrono::DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339())
                                    .unwrap()
                            })
                            .timestamp(),
                    )
            });
            descendants.sort_by(|status_a, status_b| {
                chrono::DateTime::parse_from_rfc3339(&status_a.created_at)
                    .unwrap_or_else(|_| {
                        chrono::DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339()).unwrap()
                    })
                    .timestamp()
                    .cmp(
                        &chrono::DateTime::parse_from_rfc3339(&status_b.created_at)
                            .unwrap_or_else(|_| {
                                chrono::DateTime::parse_from_rfc3339(&Utc::now().to_rfc3339())
                                    .unwrap()
                            })
                            .timestamp(),
                    )
            });
            json!({"ancestors": ancestors, "descendants": descendants})
        }
        Err(_) => json!({"error": "Status not found"}),
    }
}

pub fn favourite(pooled_connection: &PooledConnection, token: String, id: i64) -> JsonValue {
    match activity::get_activity_by_id(pooled_connection, id) {
        Ok(activity) => {
            let account: Result<Account, serde_json::Error> =
                serde_json::from_value(account_by_oauth_token(pooled_connection, token).into());
            match account {
                Ok(account) => {
                    kibou_api::react(
                        &account.id.parse::<i64>().unwrap(),
                        "Like",
                        activity.data["object"]["id"].as_str().unwrap(),
                    );
                    return status_by_id(pooled_connection, id);
                }
                Err(_) => json!({"error": "Token invalid!"}),
            }
        }
        Err(_) => json!({"error": "Status not found"}),
    }
}

pub fn follow(token: String, id: i64) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token.to_string()) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                let followee = actor::get_actor_by_id(&database, &id).unwrap();

                kibou_api::follow(&actor.actor_uri, &followee.actor_uri);
                return json!(Relationship {
                    id: followee.id.to_string(),
                    following: true,
                    followed_by: false,
                    blocking: false,
                    muting: false,
                    muting_notifications: false,
                    requested: false,
                });
            }
            Err(_) => json!({"error": "User not found."}),
        },
        Err(_) => json!({"error": "Token invalid!"}),
    }
}

pub fn home_timeline(
    pooled_connection: &PooledConnection,
    parameters: HomeTimeline,
    token: String,
) -> JsonValue {
    match verify_token(pooled_connection, token) {
        Ok(token) => {
            match actor::get_local_actor_by_preferred_username(pooled_connection, &token.actor) {
                Ok(actor) => {
                    match get_home_timeline(
                        pooled_connection,
                        actor,
                        parameters.max_id,
                        parameters.since_id,
                        parameters.min_id,
                        parameters.limit,
                    ) {
                        Ok(statuses) => cached_statuses(pooled_connection, statuses),
                        Err(_e) => {
                            json!({"error": "An error occured while generating home timeline"})
                        }
                    }
                }
                Err(_e) => json!({"error": "User associated to token not found"}),
            }
        }
        Err(_e) => json!({"error": "Invalid oauth token"}),
    }
}

pub fn instance_info() -> JsonValue {
    let database = database::establish_connection();
    json!(Instance {
        uri: format!(
            "{base_scheme}://{base_domain}",
            base_scheme = env::get_value(String::from("endpoint.base_scheme")),
            base_domain = env::get_value(String::from("endpoint.base_domain"))
        ),
        title: env::get_value(String::from("node.name")),
        description: env::get_value(String::from("node.description")),
        email: env::get_value(String::from("node.contact_email")),
        version: String::from("2.3.0 (compatible; Kibou 0.1)"),
        thumbnail: None,
        // Kibou does not support Streaming_API yet, but this value is not nullable according to
        // Mastodon-API's specifications, so that is why it is showing an empty value instead
        urls: serde_json::json!({"streaming_api": ""}),
        // `domain_count` always stays 0 as Kibou does not keep data about remote nodes
        stats: serde_json::json!({"user_count": actor::count_local_actors(&database).unwrap_or_else(|_| 0),
        "status_count": activity::count_local_ap_notes(&database).unwrap_or_else(|_| 0),
        "domain_count": 0}),
        languages: vec![],
        contact_account: None
    })
}

pub fn notifications(
    pooled_connection: &PooledConnection,
    token: String,
    limit: Option<i64>,
) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => {
            match actor::get_local_actor_by_preferred_username(pooled_connection, &token.actor) {
                Ok(actor) => {
                    match notifications_for_actor(
                        pooled_connection,
                        &actor,
                        None,
                        None,
                        None,
                        limit,
                    ) {
                        Ok(notifications) => cached_notifications(pooled_connection, notifications),
                        Err(_) => {
                            json!({"error": "An error occured while generating notifications"})
                        }
                    }
                }
                Err(_) => json!({"error": "User associated to token not found"}),
            }
        }
        Err(_) => json!({"error": "Invalid oauth token"}),
    }
}

pub fn public_timeline(
    pooled_connection: &PooledConnection,
    parameters: PublicTimeline,
) -> JsonValue {
    match get_public_timeline(
        pooled_connection,
        parameters.local.unwrap_or_else(|| false),
        parameters.only_media.unwrap_or_else(|| false),
        parameters.max_id,
        parameters.since_id,
        parameters.min_id,
        parameters.limit,
    ) {
        Ok(statuses) => cached_statuses(pooled_connection, statuses),
        Err(_e) => json!({"error": "An error occured while generating timeline."}),
    }
}

pub fn relationships(token: &str, ids: Vec<i64>) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token.to_string()) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                let mut relationships: Vec<Relationship> = Vec::new();
                let activitypub_followers: Vec<serde_json::Value> =
                    serde_json::from_value(actor.followers["activitypub"].to_owned()).unwrap();
                let activitypub_followees =
                    actor::get_actor_followees(&database, &actor.actor_uri).unwrap();

                for id in ids {
                    let follower_actor = actor::get_actor_by_id(&database, &id).unwrap();

                    match activitypub_followers.iter().position(|ref follower| {
                        follower["href"].as_str().unwrap() == follower_actor.actor_uri
                    }) {
                        Some(_) => {
                            relationships.push(Relationship {
                                id: id.to_string(),
                                following: false,
                                followed_by: true,
                                blocking: false,
                                muting: false,
                                muting_notifications: false,
                                requested: false,
                            });
                        }
                        None => {
                            relationships.push(Relationship {
                                id: id.to_string(),
                                following: false,
                                followed_by: false,
                                blocking: false,
                                muting: false,
                                muting_notifications: false,
                                requested: false,
                            });
                        }
                    }

                    match activitypub_followees
                        .iter()
                        .position(|ref followee| followee.id == id)
                    {
                        Some(_) => {
                            match relationships
                                .iter()
                                .position(|ref follower| follower.id == id.to_string())
                            {
                                Some(index) => {
                                    relationships[index].following = true;
                                }
                                None => {
                                    relationships.push(Relationship {
                                        id: id.to_string(),
                                        following: true,
                                        followed_by: false,
                                        blocking: false,
                                        muting: false,
                                        muting_notifications: false,
                                        requested: false,
                                    });
                                }
                            }
                        }
                        None => (),
                    }
                }
                return json!(relationships);
            }
            Err(_) => json!({"error": "User not found."}),
        },
        Err(_) => json!({"error": "Access token invalid!"}),
    }
}

pub fn reblog(pooled_connection: &PooledConnection, token: String, id: i64) -> JsonValue {
    match activity::get_activity_by_id(pooled_connection, id) {
        Ok(activity) => {
            let account: Result<Account, serde_json::Error> =
                serde_json::from_value(account_by_oauth_token(pooled_connection, token).into());
            match account {
                Ok(account) => {
                    kibou_api::react(
                        &account.id.parse::<i64>().unwrap(),
                        "Announce",
                        activity.data["object"]["id"].as_str().unwrap(),
                    );
                    return status_by_id(pooled_connection, id);
                }
                Err(_) => json!({"error": "Token invalid!"}),
            }
        }
        Err(_) => json!({"error": "Status not found"}),
    }
}

pub fn status_by_id(pooled_connection: &PooledConnection, id: i64) -> JsonValue {
    let statuses: Vec<Status> =
        serde_json::from_value(cached_statuses(pooled_connection, vec![id]).into())
            .unwrap_or_else(|_| Vec::new());

    if statuses.len() > 0 {
        return json!(statuses[0]);
    } else {
        return json!({"error": "Status not found"});
    }
}

pub fn status_post(
    pooled_connection: &PooledConnection,
    form: StatusForm,
    token: String,
) -> JsonValue {
    match verify_token(pooled_connection, token) {
        Ok(token) => {
            match actor::get_local_actor_by_preferred_username(pooled_connection, &token.actor) {
                Ok(actor) => {
                    let status_id = kibou_api::status_build(
                        actor.actor_uri,
                        form.status.unwrap(),
                        &form.visibility.unwrap(),
                        form.in_reply_to_id,
                    );

                    return status_by_id(pooled_connection, status_id);
                }
                Err(_) => json!({"error": "Account not found"}),
            }
        }
        Err(_) => json!({"error": "OAuth token invalid"}),
    }
}

pub fn unfollow(token: String, target_id: i64) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                let followee = actor::get_actor_by_id(&database, &target_id).unwrap();

                kibou_api::unfollow(actor.actor_uri, followee.actor_uri);
                return json!(Relationship {
                    id: followee.id.to_string(),
                    following: false,
                    followed_by: false,
                    blocking: false,
                    muting: false,
                    muting_notifications: false,
                    requested: false
                });
            }
            Err(_) => json!({"error": "User not found."}),
        },
        Err(_) => json!({"error": "Token invalid!"}),
    }
}

// This function is used to return an empty array on endpoints which are not yet implemented, this
// happens to prevent breaking Mastodon_API-compatible clients
pub fn unsupported_endpoint() -> JsonValue {
    return json!([]);
}

fn status_children_for_id(
    pooled_connection: &PooledConnection,
    id: i64,
    resolve_children: bool,
) -> Vec<Status> {
    let mut statuses: Vec<Status> = vec![];

    let head_status: Result<Status, serde_json::Error> =
        serde_json::from_value(status_by_id(pooled_connection, id).into());
    match head_status {
        Ok(status) => match get_ap_object_replies_by_id(pooled_connection, &status.uri) {
            Ok(replies) => {
                if !replies.is_empty() {
                    for reply in replies {
                        if resolve_children {
                            let mut child_statuses =
                                status_children_for_id(pooled_connection, reply.id, true);
                            statuses.append(&mut child_statuses);
                        }

                        statuses.push(Status::try_from(reply).unwrap());
                    }
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Err(e) => eprintln!("{}", e),
    }
    return statuses;
}

fn status_parents_for_id(
    pooled_connection: &PooledConnection,
    id: i64,
    is_head: bool,
) -> Vec<Status> {
    let mut statuses: Vec<Status> = vec![];

    let head_status: Result<Status, serde_json::Error> =
        serde_json::from_value(status_by_id(pooled_connection, id).into());
    match head_status {
        Ok(status) => {
            if status.in_reply_to_id.is_some() {
                statuses.append(&mut status_parents_for_id(
                    pooled_connection,
                    status
                        .in_reply_to_id
                        .clone()
                        .unwrap()
                        .parse::<i64>()
                        .unwrap(),
                    false,
                ));
            }

            if !is_head {
                statuses.append(&mut status_children_for_id(pooled_connection, id, false));
                statuses.dedup_by_key(|ref mut s| s.id == status.id);
                statuses.push(status);
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    return statuses;
}
