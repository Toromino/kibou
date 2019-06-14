use activity;
use activity::{get_ap_object_by_id, get_ap_object_replies_by_id};
use activitypub;
use actor;
use actor::get_actor_by_id;
use actor::get_actor_by_uri;
use chrono::Utc;
use database;
use diesel::PgConnection;
use env;
use kibou_api;
use mastodon_api::{
    Account, Attachment, HomeTimeline, PublicTimeline, RegistrationForm, Relationship, Source,
    Status, StatusForm,
};
use oauth;
use oauth::application::Application as OAuthApplication;
use oauth::token::{verify_token, Token};
use regex::Regex;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use timeline;
use timeline::{get_home_timeline, get_public_timeline};

pub fn account_json_by_id(id: i64) -> JsonValue {
    let database = database::establish_connection();

    match actor::get_actor_by_id(&database, id) {
        Ok(actor) => json!(serialize_account(actor, false)),
        Err(_) => json!({"error": "User not found."}),
    }
}

pub fn account_by_oauth_token(token: String) -> Result<Account, diesel::result::Error> {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => Ok(serialize_account(actor, true)),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

pub fn account_json_by_oauth_token(token: String) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => json!(serialize_account(actor, true)),
            Err(_) => json!({"error": "No user is associated to this token!"}),
        },
        Err(_) => json!({"error": "Token invalid!"}),
    }
}

pub fn account_create_json(form: &RegistrationForm) -> JsonValue {
    match account_create(form) {
        Some(token) => serde_json::to_value(token).unwrap().into(),
        None => json!({"error": "Account could not be created!"}),
    }
}

pub fn account_create(form: &RegistrationForm) -> Option<Token> {
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
        };

        actor::create_actor(&database, &mut new_actor);

        match actor::get_local_actor_by_preferred_username(&database, &form.username) {
            Ok(actor) => Some(oauth::token::create(&form.username)),
            Err(_) => None,
        }
    } else {
        return None;
    }
}

pub fn account_statuses_json_by_id(
    id: i64,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> JsonValue {
    let database = database::establish_connection();

    match actor::get_actor_by_id(&database, id) {
        Ok(actor) => {
            match timeline::get_user_timeline(&database, actor, max_id, since_id, min_id, limit) {
                Ok(statuses) => {
                    let mut serialized_statuses: Vec<Status> = vec![];

                    for status in statuses {
                        if let Ok(valid_status) = status_cached_by_id(status) {
                            serialized_statuses.push(valid_status)
                        }
                    }
                    json!(serialized_statuses)
                }
                Err(_) => json!({"error": "Error generating user timeline."}),
            }
        }
        Err(_) => json!({"error": "User not found."}),
    }
}

pub fn application_create(application: OAuthApplication) -> rocket_contrib::json::JsonValue {
    let database = database::establish_connection();
    let oauth_app: OAuthApplication = oauth::application::create(&database, application);
    rocket_contrib::json!({
        "name": oauth_app.client_name.unwrap_or_default(),
        "website": oauth_app.website,
        "client_id": oauth_app.client_id,
        "client_secret": oauth_app.client_secret,
        "redirect_uri": oauth_app.redirect_uris,
        "id": oauth_app.id
    })
}

pub fn context_json_for_id(id: i64) -> JsonValue {
    let database = database::establish_connection();

    match activity::get_activity_by_id(&database, id) {
        Ok(activity) => {
            json!({"ancestors": status_parents_for_id(&database, id, true), "descendants": status_children_for_id(&database, id, true)})
        }
        Err(_) => json!({"error": "Status not found"}),
    }
}

pub fn follow_json(token: String, id: i64) -> JsonValue {
    match follow(&token, id) {
        Ok(relationship) => json!(relationship),
        Err(e) => json!({ "error": e }),
    }
}

pub fn follow(token: &str, target_id: i64) -> Result<Relationship, &'static str> {
    let database = database::establish_connection();

    match verify_token(&database, token.to_string()) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                let followee = actor::get_actor_by_id(&database, target_id).unwrap();

                kibou_api::follow(actor.actor_uri, followee.actor_uri);
                return Ok(Relationship {
                    id: followee.id.to_string(),
                    following: true,
                    followed_by: false,
                    blocking: false,
                    muting: false,
                    muting_notifications: false,
                    requested: false,
                });
            }
            Err(_) => Err("User not found."),
        },
        Err(_) => Err("Token invalid!"),
    }
}

pub fn home_timeline_json(parameters: HomeTimeline, token: String) -> JsonValue {
    match home_timeline(parameters, token) {
        Ok(statuses) => json!(statuses),
        Err(_) => json!({"error": "An error occured while generating timeline."}),
    }
}

pub fn public_timeline_json(parameters: PublicTimeline) -> JsonValue {
    match public_timeline(parameters) {
        Ok(statuses) => json!(statuses),
        Err(_) => json!({"error": "An error occured while generating timeline."}),
    }
}

pub fn relationships_json_by_token(token: &str, ids: Vec<i64>) -> JsonValue {
    match relationships_by_token(token, ids) {
        Ok(relationships) => json!(relationships),
        Err(e) => json!({ "error": e }),
    }
}

pub fn relationships_by_token(
    token: &str,
    ids: Vec<i64>,
) -> Result<Vec<Relationship>, &'static str> {
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
                    let follower_actor = actor::get_actor_by_id(&database, id).unwrap();

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
                return Ok(relationships);
            }
            Err(_) => Err("User not found."),
        },
        Err(_) => Err("Acces token invalid!"),
    }
}

pub fn serialize_account(mut actor: actor::Actor, include_source: bool) -> Account {
    let database = database::establish_connection();

    let mut new_account = Account {
        id: actor.id.to_string(),
        username: actor.preferred_username.clone(),
        acct: actor.get_acct(),
        display_name: actor.username.unwrap_or_else(|| String::from("")),
        locked: false,
        created_at: actor.created.to_string(),
        followers_count: count_followers(&database, &actor.id),
        following_count: count_followees(&database, &actor.id),
        statuses_count: count_statuses(&database, &actor.actor_uri),
        note: actor.summary.unwrap_or_else(|| String::from("")),
        url: actor.actor_uri,
        avatar: actor.icon.clone().unwrap_or_else(|| {
            format!(
                "{}://{}/static/assets/default_avatar.png",
                env::get_value(String::from("endpoint.base_scheme")),
                env::get_value(String::from("endpoint.base_domain"))
            )
        }),
        avatar_static: actor.icon.unwrap_or_else(|| {
            format!(
                "{}://{}/static/assets/default_avatar.png",
                env::get_value(String::from("endpoint.base_scheme")),
                env::get_value(String::from("endpoint.base_domain"))
            )
        }),
        header: format!(
            "{}://{}/static/assets/default_banner.png",
            env::get_value(String::from("endpoint.base_scheme")),
            env::get_value(String::from("endpoint.base_domain"))
        ),
        header_static: format!(
            "{}://{}/static/assets/default_banner.png",
            env::get_value(String::from("endpoint.base_scheme")),
            env::get_value(String::from("endpoint.base_domain"))
        ),
        emojis: vec![],
        source: None,
    };

    if include_source {
        new_account.source = Some(Source {
            privacy: None,
            sensitive: None,
            language: None,
            note: new_account.note.clone(),
            fields: None,
        });
    }

    return new_account;
}

pub fn serialize_status(activity: activity::Activity) -> Result<Status, ()> {
    serialize_status_from_activitystreams(activity)
}

pub fn status_json_by_id(id: i64) -> JsonValue {
    let database = database::establish_connection();

    match status_cached_by_id(id) {
        Ok(status) => json!(status),
        Err(_) => json!({"error": "Status not found."}),
    }
}

pub fn status_post(form: StatusForm, token: String) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                let status_id = kibou_api::status_build(
                    actor.actor_uri,
                    form.status.unwrap(),
                    &form.visibility.unwrap(),
                    form.in_reply_to_id,
                );

                return json!(status_cached_by_id(status_id));
            }
            Err(_) => json!({"error": "Account not found"}),
        },
        Err(_) => json!({"error": "OAuth token invalid"}),
    }
}

pub fn unfollow(token: String, target_id: i64) -> JsonValue {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                let followee = actor::get_actor_by_id(&database, target_id).unwrap();

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

cached! {
    MASTODON_API_ACCOUNT_CACHE;
fn account_by_uri(uri: &'static str) -> Result<serde_json::Value, String> = {
    let database = database::establish_connection();
    match actor::get_actor_by_uri(&database, uri) {
        Ok(account) => {
                Ok(serde_json::to_value(serialize_account(account, false)).unwrap())
            },
        Err(_) => Err(format!("Account not found: {}", &uri)),
    }
}
}

fn account_cached_by_uri(uri: &'static str) -> Result<Account, String> {
    match account_by_uri(uri) {
        Ok(account) => Ok(serde_json::from_value(account).unwrap()),
        Err(e) => Err(e),
    }
}

fn count_favourites(database: &PgConnection, status_id: &str) -> i64 {
    match activity::count_ap_object_reactions_by_id(database, status_id, "Like") {
        Ok(replies) => replies as i64,
        Err(_) => 0,
    }
}

fn count_followees(db_connection: &PgConnection, account_id: &i64) -> i64 {
    match actor::count_followees(db_connection, *account_id) {
        Ok(followees) => followees as i64,
        Err(_) => 0,
    }
}

fn count_followers(db_connection: &PgConnection, account_id: &i64) -> i64 {
    match get_actor_by_id(db_connection, *account_id) {
        Ok(actor) => {
            let activitypub_followers: Vec<serde_json::Value> =
                serde_json::from_value(actor.followers["activitypub"].to_owned())
                    .unwrap_or_else(|_| Vec::new());
            return activitypub_followers.len() as i64;
        }
        Err(_) => 0,
    }
}

fn count_reblogs(database: &PgConnection, status_id: &str) -> i64 {
    match activity::count_ap_object_reactions_by_id(database, status_id, "Announce") {
        Ok(replies) => replies as i64,
        Err(_) => 0,
    }
}

fn count_replies(database: &PgConnection, status_id: &str) -> i64 {
    match activity::count_ap_object_replies_by_id(database, status_id) {
        Ok(replies) => replies as i64,
        Err(_) => 0,
    }
}

fn count_statuses(db_connection: &PgConnection, account_uri: &str) -> i64 {
    match activity::count_ap_notes_for_actor(db_connection, account_uri) {
        Ok(statuses) => statuses as i64,
        Err(_) => 0,
    }
}

fn home_timeline(
    parameters: HomeTimeline,
    token: String,
) -> Result<Vec<Status>, diesel::result::Error> {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, &token.actor) {
            Ok(actor) => {
                match get_home_timeline(
                    &database,
                    actor,
                    parameters.max_id,
                    parameters.since_id,
                    parameters.min_id,
                    parameters.limit,
                ) {
                    Ok(statuses) => {
                        let mut serialized_statuses: Vec<Status> = vec![];

                        for status in statuses {
                            if let Ok(valid_status) = status_cached_by_id(status) {
                                serialized_statuses.push(valid_status)
                            }
                        }

                        Ok(serialized_statuses)
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

fn public_timeline(parameters: PublicTimeline) -> Result<Vec<Status>, diesel::result::Error> {
    let database = database::establish_connection();

    match get_public_timeline(
        &database,
        parameters.local.unwrap_or_else(|| false),
        parameters.only_media.unwrap_or_else(|| false),
        parameters.max_id,
        parameters.since_id,
        parameters.min_id,
        parameters.limit,
    ) {
        Ok(statuses) => {
            let mut serialized_statuses: Vec<Status> = vec![];

            for status in statuses {
                if let Ok(valid_status) = status_cached_by_id(status) {
                    serialized_statuses.push(valid_status)
                }
            }

            Ok(serialized_statuses)
        }
        Err(e) => Err(e),
    }
}

fn serialize_attachments_from_activitystreams(activity: &activity::Activity) -> Vec<Attachment> {
    let mut media_attachments: Vec<Attachment> = Vec::new();
    match activity.data["object"].get("attachment") {
        Some(attachmenets) => {
            let serialized_attachments: Vec<activitypub::Attachment> =
                serde_json::from_value(activity.data["object"]["attachment"].to_owned()).unwrap();

            for attachment in serialized_attachments {
                media_attachments.push(Attachment {
                    id: attachment
                        .name
                        .unwrap_or_else(|| String::from("Unnamed attachment")),
                    _type: String::from("image"),
                    url: attachment.url.clone(),
                    remote_url: Some(attachment.url.clone()),
                    preview_url: attachment.url,
                    text_url: None,
                    meta: None,
                    description: attachment.content,
                });
            }
        }
        None => (),
    }
    return media_attachments;
}

fn serialize_status_from_activitystreams(activity: activity::Activity) -> Result<Status, ()> {
    let database = database::establish_connection();
    let serialized_attachments: Vec<Attachment> =
        serialize_attachments_from_activitystreams(&activity);
    let serialized_activity: activitypub::activity::Activity =
        serde_json::from_value(activity.data).unwrap();
    let serialized_account =
        account_cached_by_uri(Box::leak(activity.actor.into_boxed_str())).unwrap();

    match serialized_activity._type.as_str() {
        "Create" => {
            let serialized_object: activitypub::activity::Object =
                serde_json::from_value(serialized_activity.object).unwrap();
            let mut parent_object: Option<String>;
            let mut parent_object_account: Option<String>;

            match serialized_object.inReplyTo {
                Some(object) => match get_ap_object_by_id(&database, &object) {
                    Ok(parent_activity) => {
                        parent_object = Some(parent_activity.id.to_string());

                        match get_actor_by_uri(&database, &parent_activity.actor) {
                            Ok(parent_actor) => {
                                parent_object_account = Some(parent_actor.id.to_string())
                            }
                            Err(_) => parent_object_account = None,
                        }
                    }
                    Err(_) => {
                        parent_object = None;
                        parent_object_account = None;
                    }
                },
                None => {
                    parent_object = None;
                    parent_object_account = None;
                }
            }

            Ok(Status {
                id: activity.id.to_string(),
                uri: serialized_object.id.clone(),
                url: Some(serialized_object.id.clone()),
                account: serialized_account,
                in_reply_to_id: parent_object,
                in_reply_to_account_id: parent_object_account,
                reblog: None,
                content: serialized_object.content,
                created_at: serialized_object.published,
                emojis: vec![],
                replies_count: count_replies(&database, &serialized_object.id),
                reblogs_count: count_reblogs(&database, &serialized_object.id),
                favourites_count: count_favourites(&database, &serialized_object.id),
                reblogged: Some(false),
                favourited: Some(false),
                muted: Some(false),
                sensitive: serialized_object.sensitive.unwrap_or_else(|| false),
                spoiler_text: String::new(),
                visibility: String::from("public"),
                media_attachments: serialized_attachments,
                mentions: vec![],
                tags: vec![],
                application: serde_json::json!({"name": "Web", "website": null}),
                language: None,
                pinned: None,
            })
        }
        "Announce" => {
            match get_ap_object_by_id(&database, serialized_activity.object.as_str().unwrap()) {
                Ok(reblog) => {
                    let serialized_reblog: Status =
                        serialize_status_from_activitystreams(reblog).unwrap();

                    Ok(Status {
                        id: activity.id.to_string(),
                        uri: serialized_activity.id.clone(),
                        url: Some(serialized_activity.id.clone()),
                        account: serialized_account,
                        in_reply_to_id: None,
                        in_reply_to_account_id: None,
                        reblog: Some(serde_json::to_value(serialized_reblog).unwrap()),
                        content: String::from("reblog"),
                        created_at: serialized_activity.published,
                        emojis: vec![],
                        replies_count: 0,
                        reblogs_count: 0,
                        favourites_count: 0,
                        reblogged: Some(false),
                        favourited: Some(false),
                        muted: Some(false),
                        sensitive: false,
                        spoiler_text: String::new(),
                        visibility: String::from("public"),
                        media_attachments: vec![],
                        mentions: vec![],
                        tags: vec![],
                        application: serde_json::json!({"name": "Web", "website": null}),
                        language: None,
                        pinned: None,
                    })
                }
                Err(e) => Err(()),
            }
        }
        _ => Err(()),
    }
}

cached! {
    MASTODON_API_STATUS_CACHE;
fn status_by_id(id: i64) -> Result<serde_json::Value, String> = {
    let database = database::establish_connection();
    match activity::get_activity_by_id(&database, id) {
        Ok(activity) => {
            match serialize_status(activity)
            {
                Ok(serialized_status) => Ok(serde_json::to_value(serialized_status).unwrap()),
                Err(_) => Err(format!("Failed to serialize status:"))
            }
            },
        Err(_) => Err(format!("Status not found: {}", &id)),
    }
}
}

fn status_cached_by_id(id: i64) -> Result<Status, String> {
    match status_by_id(id) {
        Ok(status) => Ok(serde_json::from_value(status).unwrap()),
        Err(e) => Err(e),
    }
}

fn status_children_for_id(
    db_connection: &PgConnection,
    id: i64,
    resolve_children: bool,
) -> Vec<Status> {
    let mut statuses: Vec<Status> = vec![];

    match status_cached_by_id(id) {
        Ok(status) => match get_ap_object_replies_by_id(&db_connection, &status.uri) {
            Ok(replies) => {
                if !replies.is_empty() {
                    for reply in replies {
                        if resolve_children {
                            let mut child_statuses =
                                status_children_for_id(&db_connection, reply.id, true);
                            statuses.append(&mut child_statuses);
                        }

                        statuses.push(serialize_status(reply).unwrap());
                    }
                }
            }
            Err(e) => eprintln!("{}", e),
        },
        Err(e) => eprintln!("{}", e),
    }
    return statuses;
}

fn status_parents_for_id(db_connection: &PgConnection, id: i64, is_head: bool) -> Vec<Status> {
    let mut statuses: Vec<Status> = vec![];

    match status_cached_by_id(id) {
        Ok(status) => {
            if status.in_reply_to_id.is_some() {
                statuses.append(&mut status_parents_for_id(
                    &db_connection,
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
                statuses.append(&mut status_children_for_id(db_connection, id, false));
                statuses.dedup_by_key(|ref mut s| s.id == status.id);
                statuses.push(status);
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    return statuses;
}
