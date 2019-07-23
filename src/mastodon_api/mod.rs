pub mod controller;
pub mod routes;
use activity::{
    count_ap_notes_for_actor, count_ap_object_reactions_by_id, count_ap_object_replies_by_id,
    get_ap_object_by_id, Activity,
};
use activitypub;
use actor::{count_followees, get_actor_by_uri, Actor};
use database;
use database::PooledConnection;
use env;
use rocket::request;
use rocket::request::FromRequest;
use rocket::request::Request;
use rocket::Outcome;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize)]
pub struct Account {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#account
    pub id: String,
    pub username: String,
    pub acct: String,
    pub display_name: String,
    pub locked: bool,
    pub created_at: String,
    pub followers_count: i64,
    pub following_count: i64,
    pub statuses_count: i64,
    pub note: String,
    pub url: String,
    pub avatar: String,
    pub avatar_static: String,
    pub header: String,
    pub header_static: String,
    pub emojis: Vec<Emoji>,
    pub source: Option<Source>,
}

#[derive(FromForm)]
pub struct ApplicationForm {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#application
    // - https://docs.joinmastodon.org/api/rest/apps/#post-api-v1-apps
    pub client_name: String,
    pub redirect_uris: String,
    pub scopes: String,
    pub website: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub url: String,
    pub remote_url: Option<String>,
    pub preview_url: String,
    pub text_url: Option<String>,
    pub meta: Option<serde_json::Value>,
    pub description: Option<String>,
}

#[derive(Debug)]
pub struct AuthorizationHeader(pub String);

#[derive(Serialize, Deserialize)]
pub struct Emoji {
    pub shortcode: String,
    pub static_url: String,
    pub url: String,
    pub visible_in_picker: bool,
}

#[derive(FromForm)]
pub struct HomeTimeline {
    pub max_id: Option<i64>,
    pub since_id: Option<i64>,
    pub min_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct Instance {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#instance
    pub uri: String,
    pub title: String,
    pub description: String,
    pub email: String,
    pub version: String,
    pub thumbnail: Option<String>,
    pub urls: serde_json::Value,
    pub stats: serde_json::Value,
    pub languages: Vec<String>,
    pub contact_account: Option<Account>,
}

#[derive(Serialize, Deserialize)]
pub struct Mention {
    pub url: String,
    pub username: String,
    pub acct: String,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub created_at: String,
    pub account: Account,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<Status>,
}

#[derive(FromForm)]
pub struct PublicTimeline {
    pub local: Option<bool>,
    pub only_media: Option<bool>,
    pub max_id: Option<i64>,
    pub since_id: Option<i64>,
    pub min_id: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(FromForm)]
pub struct RegistrationForm {
    // Properties acctording to
    // - https://docs.joinmastodon.org/api/rest/accounts/#post-api-v1-accounts
    pub username: String,
    pub email: String,
    pub password: String,
    // Optional values in Kibou, as they're not used by the backend (yet?)
    pub agreement: Option<String>,
    pub locale: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Relationship {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#relationship
    pub id: String,
    pub following: bool,
    pub followed_by: bool,
    pub blocking: bool,
    pub muting: bool,
    pub muting_notifications: bool,
    pub requested: bool,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Source {
    pub privacy: Option<String>,
    pub sensitive: Option<bool>,
    pub language: Option<String>,
    pub note: String,
    pub fields: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct Status {
    // Properties according to
    // - https://docs.joinmastodon.org/api/entities/#status
    pub id: String,
    pub uri: String,
    pub url: Option<String>,
    pub account: Account,
    pub in_reply_to_id: Option<String>,
    pub in_reply_to_account_id: Option<String>,
    pub reblog: Option<serde_json::Value>,
    pub content: String,
    pub created_at: String,
    pub emojis: Vec<Emoji>,
    pub replies_count: i64,
    pub reblogs_count: i64,
    pub favourites_count: i64,
    pub reblogged: Option<bool>,
    pub favourited: Option<bool>,
    pub muted: Option<bool>,
    pub sensitive: bool,
    pub spoiler_text: String,
    pub visibility: String,
    pub media_attachments: Vec<Attachment>,
    pub mentions: Vec<Mention>,
    pub tags: Vec<Tag>,
    pub application: serde_json::Value,
    pub language: Option<String>,
    pub pinned: Option<bool>,
}

#[derive(FromForm)]
pub struct StatusForm {
    pub status: Option<String>,
    pub in_reply_to_id: Option<String>,
    pub media_ids: Option<String>,
    pub sensitive: Option<bool>,
    pub spoiler_text: Option<String>,
    pub visibility: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Tag {
    pub name: String,
    pub url: String,
    pub history: Option<serde_json::Value>,
}

// Entities throw an empty error if they fail serializing. Entities should only fail silently, as
// these errors do not show up on the API. Furthermore should serializing a Mastodon-API entity
// never panic.

impl Account {
    pub fn from_actor(
        pooled_connection: &PooledConnection,
        mut actor: Actor,
        include_source: bool,
    ) -> Account {
        let followees = count_followees(&pooled_connection, actor.id).unwrap_or_else(|_| 0) as i64;
        let followers: Vec<serde_json::Value> =
            serde_json::from_value(actor.followers["activitypub"].to_owned())
                .unwrap_or_else(|_| Vec::new());

        let statuses = count_ap_notes_for_actor(&pooled_connection, &actor.actor_uri)
            .unwrap_or_else(|_| 0) as i64;

        let mut new_account = Account {
            id: actor.id.to_string(),
            username: actor.preferred_username.clone(),
            acct: actor.get_acct(),
            display_name: actor.username.unwrap_or_else(|| String::from("")),
            locked: false,
            created_at: actor.created.to_string(),
            followers_count: followers.len() as i64,
            following_count: followees,
            statuses_count: statuses,
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
}

impl Notification {
    pub fn try_from(activity: Activity) -> Result<Self, ()> {
        let activitypub_activity: Result<activitypub::activity::Activity, serde_json::Error> =
            serde_json::from_value(activity.data);
        let pooled_connection = &PooledConnection(database::POOL.get().unwrap());

        match activitypub_activity {
            Ok(activity_inner) => {
                return Notification::from_activitystreams(
                    pooled_connection,
                    activity.id,
                    activity_inner,
                )
            }
            Err(_) => return Err(()),
        };
    }

    fn from_activitystreams(
        pooled_connection: &PooledConnection,
        activity_id: i64,
        activity: activitypub::activity::Activity,
    ) -> Result<Self, ()> {
        let account_result: Result<Account, serde_json::Error> = serde_json::from_value(
            controller::cached_account(
                pooled_connection,
                Box::leak(activity.actor.to_owned().into_boxed_str()),
            )
            .into(),
        );

        // An activity without an actor should never occur, but if it does,
        // this function needs to fail.
        match account_result {
            Ok(account) => {
                let notification_type = match activity._type.as_str() {
                    "Follow" => String::from("follow"),
                    "Create" => String::from("mention"),
                    "Announce" => String::from("reblog"),
                    "Like" => String::from("favourite"),
                    _ => String::from(""),
                };
                let mut status: Option<Status> = None;

                if !notification_type.is_empty() {
                    if &notification_type == "reblog" || &notification_type == "favourite" {
                        status = serde_json::from_value(
                            controller::status_by_id(
                                pooled_connection,
                                get_ap_object_by_id(
                                    pooled_connection,
                                    activity.object.as_str().unwrap(),
                                )
                                .unwrap()
                                .id,
                            )
                            .into(),
                        )
                        .unwrap_or_else(|_| None);
                    } else if &notification_type == "mention" {
                        status = serde_json::from_value(
                            controller::status_by_id(pooled_connection, activity_id).into(),
                        )
                        .unwrap_or_else(|_| None);
                    }

                    return Ok(Notification {
                        id: activity_id.to_string(),
                        _type: notification_type,
                        created_at: activity.published,
                        account: account,
                        status: status,
                    });
                } else {
                    return Err(());
                }
            }
            Err(_) => Err(()),
        }
    }
}

impl Status {
    pub fn try_from(activity: Activity) -> Result<Self, ()> {
        let activitypub_activity: Result<activitypub::activity::Activity, serde_json::Error> =
            serde_json::from_value(activity.data);
        let pooled_connection = &PooledConnection(database::POOL.get().unwrap());

        match activitypub_activity {
            Ok(activity_inner) => {
                return Status::from_activitystreams(pooled_connection, activity.id, activity_inner)
            }
            Err(_) => return Err(()),
        };
    }
    fn from_activitystreams(
        pooled_connection: &PooledConnection,
        activity_id: i64,
        activity: activitypub::activity::Activity,
    ) -> Result<Self, ()> {
        let account_result: Result<Account, serde_json::Error> = serde_json::from_value(
            controller::cached_account(pooled_connection, &activity.actor).into(),
        );

        // An activity without an actor should never occur, but if it does,
        // this function needs to fail.
        match account_result {
            Ok(account) => {
                let mut mentions: Vec<Mention> = Vec::new();
                for actor in &activity.to {
                    let mention_account: Result<Account, serde_json::Error> =
                        serde_json::from_value(
                            controller::cached_account(pooled_connection, &actor).into(),
                        );
                    // Just unwrapping every account would mean that an entire status fails serializing,
                    // because of one invalid account.
                    match mention_account {
                        Ok(mention_account) => mentions.push(Mention {
                            url: mention_account.url,
                            username: mention_account.username,
                            acct: mention_account.acct,
                            id: mention_account.id,
                        }),
                        Err(_) => (),
                    };
                }

                // The 'Public' and 'Unlisted' scope can be easily determined by the existence of
                // `https://www.w3.org/ns/activitystreams#Public` in either the 'to' or 'cc' field.
                //
                // Note that different formats like `as:Public` have already been normalized to
                // `https://www.w3.org/ns/activitystreams#Public` in activitypub::validator.
                let visibility = if activity
                    .to
                    .contains(&"https://www.w3.org/ns/activitystreams#Public".to_string())
                {
                    String::from("public")
                } else if activity
                    .cc
                    .contains(&"https://www.w3.org/ns/activitystreams#Public".to_string())
                {
                    String::from("unlisted")
                // XX - This might cause issues, as the 'Followers' endpoint of remote actors might differ
                // from Kibou's schema. But as of now Kibou does not keep track of that endpoint.
                } else if activity.to.contains(&format!("{}/followers", account.url)) {
                    String::from("private")
                } else {
                    String::from("direct")
                };

                match activity._type.as_str() {
                    "Create" => {
                        let inner_object_result: Result<
                            activitypub::activity::Object,
                            serde_json::Error,
                        > = serde_json::from_value(activity.object.clone());
                        match inner_object_result {
                            Ok(inner_object) => {
                                let mut in_reply_to: Option<String> = None;
                                let mut in_reply_to_account: Option<String> = None;
                                if inner_object.inReplyTo.is_some() {
                                    in_reply_to = match get_ap_object_by_id(
                                        pooled_connection,
                                        &inner_object.inReplyTo.unwrap(),
                                    ) {
                                        Ok(parent_activity) => {
                                            in_reply_to_account = match get_actor_by_uri(
                                                pooled_connection,
                                                &parent_activity.data["actor"].as_str().unwrap(),
                                            ) {
                                                Ok(parent_actor) => {
                                                    Some(parent_actor.id.to_string())
                                                }
                                                Err(_) => None,
                                            };
                                            Some(parent_activity.id.to_string())
                                        }
                                        Err(_) => None,
                                    };
                                }

                                let mut media_attachments: Vec<Attachment> = Vec::new();
                                match activity.object.get("attachment") {
                                    Some(_attachments) => {
                                        let attachments: Vec<activitypub::Attachment> =
                                            serde_json::from_value(
                                                activity.object["attachment"].to_owned(),
                                            )
                                            .unwrap_or_else(|_| Vec::new());

                                        for attachment in attachments {
                                            media_attachments.push(Attachment {
                                                id: attachment.name.unwrap_or_else(|| {
                                                    String::from("Unnamed attachment")
                                                }),
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

                                let favourites = count_ap_object_reactions_by_id(
                                    pooled_connection,
                                    &inner_object.id,
                                    "Like",
                                )
                                .unwrap_or_else(|_| 0)
                                    as i64;
                                let reblogs = count_ap_object_reactions_by_id(
                                    pooled_connection,
                                    &inner_object.id,
                                    "Announce",
                                )
                                .unwrap_or_else(|_| 0)
                                    as i64;
                                let replies = count_ap_object_replies_by_id(
                                    pooled_connection,
                                    &inner_object.id,
                                )
                                .unwrap_or_else(|_| 0)
                                    as i64;
                                return Ok(Status {
                                    id: activity_id.to_string(),
                                    uri: inner_object.id.clone(),
                                    url: Some(inner_object.id.clone()),
                                    account: account,
                                    in_reply_to_id: in_reply_to,
                                    in_reply_to_account_id: in_reply_to_account,
                                    reblog: None,
                                    content: inner_object.content,
                                    created_at: inner_object.published,
                                    emojis: vec![],
                                    replies_count: replies,
                                    reblogs_count: reblogs,
                                    favourites_count: favourites,
                                    reblogged: Some(false),
                                    favourited: Some(false),
                                    muted: None,
                                    sensitive: inner_object.sensitive.unwrap_or_else(|| false),
                                    spoiler_text: "".to_string(),
                                    visibility: visibility,
                                    media_attachments: media_attachments,
                                    mentions: mentions,
                                    tags: vec![],
                                    application: serde_json::json!({"name": "Web", "website": null}),
                                    language: None,
                                    pinned: None,
                                });
                            }
                            Err(_) => Err(()),
                        }
                    }
                    "Announce" => match activity.object.as_str() {
                        Some(object_id) => {
                            match get_ap_object_by_id(&pooled_connection, object_id) {
                                Ok(reblog) => match Status::try_from(reblog) {
                                    Ok(serialized_reblog) => {
                                        return Ok(Status {
                                            id: activity_id.to_string(),
                                            uri: activity.id.clone(),
                                            url: Some(activity.id.clone()),
                                            account: account,
                                            in_reply_to_id: None,
                                            in_reply_to_account_id: None,
                                            reblog: Some(
                                                serde_json::to_value(serialized_reblog).unwrap(),
                                            ),
                                            content: String::from("reblog"),
                                            created_at: activity.published,
                                            emojis: vec![],
                                            replies_count: 0,
                                            reblogs_count: 0,
                                            favourites_count: 0,
                                            reblogged: Some(false),
                                            favourited: Some(false),
                                            muted: Some(false),
                                            sensitive: false,
                                            spoiler_text: String::new(),
                                            visibility: visibility,
                                            media_attachments: vec![],
                                            mentions: vec![],
                                            tags: vec![],
                                            application: serde_json::json!({"name": "Web", "website": null}),
                                            language: None,
                                            pinned: None,
                                        })
                                    }
                                    Err(_) => Err(()),
                                },
                                Err(_) => Err(()),
                            }
                        }
                        None => Err(()),
                    },
                    _ => Err(()),
                }
            }
            Err(_) => Err(()),
        }
    }
}

impl ToString for AuthorizationHeader {
    fn to_string(&self) -> String {
        format!("{:?}", &self)
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for AuthorizationHeader {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<AuthorizationHeader, ()> {
        let headers: Vec<_> = request.headers().get("Authorization").collect();
        if headers.is_empty() {
            return Outcome::Failure((rocket::http::Status::BadRequest, ()));
        } else {
            return Outcome::Success(AuthorizationHeader(headers[0].to_string()));
        }
    }
}

lazy_static! {
    static ref MASTODON_API_ACCOUNT_CACHE: Arc<Mutex<lru::LruCache<String, serde_json::Value>>> =
        Arc::new(Mutex::new(lru::LruCache::new(400)));
    static ref MASTODON_API_NOTIFICATION_CACHE: Arc<Mutex<lru::LruCache<i64, serde_json::Value>>> =
        Arc::new(Mutex::new(lru::LruCache::new(400)));
    static ref MASTODON_API_STATUS_CACHE: Arc<Mutex<lru::LruCache<i64, serde_json::Value>>> =
        Arc::new(Mutex::new(lru::LruCache::new(400)));
}

pub fn parse_authorization_header(header: &str) -> String {
    let header_vec: Vec<&str> = header.split(" ").collect();

    return header_vec[1].replace("\")", "").to_string();
}
