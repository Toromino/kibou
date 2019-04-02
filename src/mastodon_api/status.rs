use activity::get_ap_object_by_id;
use actor;
use actor::get_actor_by_uri;
use database;
use diesel::pg::PgConnection;
use kibou_api;
use mastodon_api::account::serialize as serialize_account;
use mastodon_api::account::Account;
use oauth::token::verify_token;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};
use {activity, activitypub};

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

#[derive(Serialize, Deserialize)]
pub struct Emoji {
    pub shortcode: String,
    pub static_url: String,
    pub url: String,
    pub visible_in_picker: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Mention {
    pub url: String,
    pub username: String,
    pub acct: String,
    pub id: String,
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
    //pub card: Option<Card>,
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

pub fn get_json_by_id(id: i64) -> JsonValue {
    let database = database::establish_connection();

    match activity::get_activity_by_id(&database, id) {
        Ok(activity) => json!(serialize(activity)),
        Err(_) => json!({"error": "Status not found."}),
    }
}

pub fn serialize(activity: activity::Activity) -> Result<Status, ()> {
    serialize_from_activitystreams(activity)
}

fn serialize_from_activitystreams(activity: activity::Activity) -> Result<Status, ()> {
    let database = database::establish_connection();
    let serialized_activity: activitypub::activity::Activity =
        serde_json::from_value(activity.data).unwrap();
    let serialized_account: Account = serialize_account(
        actor::get_actor_by_uri(&database, &serialized_activity.actor).unwrap(),
        false,
    );

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
                replies_count: get_reply_count(&database, &serialized_object.id),
                reblogs_count: get_reblogs_count(&database, &serialized_object.id),
                favourites_count: get_favourite_count(&database, &serialized_object.id),
                reblogged: Some(false),
                favourited: Some(false),
                muted: Some(false),
                sensitive: true,
                spoiler_text: String::new(),

                // * Note *
                // Currently this marks all statuses as public, which should be changed in the
                // future. But this is okay for now, as long as non-public statuses are rejected
                // anyway (due to the lack of HTTP signature validation)
                visibility: String::from("public"),
                media_attachments: vec![],
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
                    let serialized_reblog: Status = serialize_from_activitystreams(reblog).unwrap();

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

pub fn post_status(form: StatusForm, token: String) {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, token.actor) {
            Ok(actor) => {
                kibou_api::status::build(
                    actor.actor_uri,
                    form.status.unwrap(),
                    &form.visibility.unwrap(),
                    form.in_reply_to_id,
                );
            }
            Err(e) => eprintln!("{}", e),
        },
        Err(e) => eprintln!("{}", e),
    }
}

fn get_reply_count(database: &PgConnection, status_id: &str) -> i64 {
    match activity::count_ap_object_replies_by_id(database, status_id) {
        Ok(replies) => replies as i64,
        Err(_) => 0,
    }
}

fn get_favourite_count(database: &PgConnection, status_id: &str) -> i64 {
    match activity::count_ap_object_reactions_by_id(database, status_id, "Like") {
        Ok(replies) => replies as i64,
        Err(_) => 0,
    }
}

fn get_reblogs_count(database: &PgConnection, status_id: &str) -> i64 {
    match activity::count_ap_object_reactions_by_id(database, status_id, "Announce") {
        Ok(replies) => replies as i64,
        Err(_) => 0,
    }
}
