//! Kibou_API provides a common application layer between data reprentations of modules such as
//! ActivityPub, Mastodon_API and internal reprentations.
//!
//! Furthermore Kibou_API will implement endpoints which are unique to the Kibou backend.
//!

pub mod routes;

use activity::{
    get_activity_by_id, get_ap_activity_by_id, get_ap_object_by_id, insert_activity,
    type_exists_for_object_id,
};
use activitypub;
use activitypub::{add_follow, remove_follow};
use activitypub::controller as ap_controller;
use activitypub::Tag;
use actor::{get_actor_by_acct, get_actor_by_id, get_actor_by_uri, is_actor_followed_by, Actor};
use database;
use database::{Pool, PooledConnection};
use diesel::PgConnection;
use html;
use mastodon_api;
use notification::{self, Notification};
use regex::Regex;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use std::thread;
use timeline;
use web::federator;

pub fn follow(pooled_connection: &PooledConnection, sender: &str, receipient: &str) {
    let serialized_actor: Actor = get_actor_by_uri(pooled_connection, sender).unwrap();

    if sender != receipient {
        match get_actor_by_uri(pooled_connection, &receipient) {
            Ok(followee) => {
                if !is_actor_followed_by(pooled_connection, &followee.actor_uri, sender).unwrap() {
                    let follow_activity = activitypub::Activity::new(
                        "Follow",
                        sender,
                        serde_json::json!(receipient),
                        vec![receipient.to_string()],
                        Vec::new(),
                    );

                    let activity = insert_activity(pooled_connection, follow_activity.clone().into());

                    if !followee.local {
                        federator::enqueue(
                            serialized_actor,
                            serde_json::json!(&follow_activity),
                            vec![followee.inbox.unwrap()],
                        );
                    } else {
                        add_follow(receipient, sender, &follow_activity.id);
                        notification::insert(
                            pooled_connection,
                            Notification::new(activity.id, followee.id),
                        );
                    }
                }
            }
            Err(_) => (),
        }
    }
}

pub fn react(pooled_connection: &PooledConnection, actor: &i64, _type: &str, object_id: &str) {
    let serialized_actor: Actor = get_actor_by_id(pooled_connection, actor)
        .expect("Actor should exist!");

    if !type_exists_for_object_id(pooled_connection, _type, &serialized_actor.actor_uri, object_id)
        .unwrap_or_else(|_| true)
    {
        match get_ap_object_by_id(pooled_connection, object_id) {
            Ok(activity) => {
                let ap_activity = activitypub::Activity::from(activity);

                let mut to: Vec<String> = ap_activity.to.clone();
                let mut cc: Vec<String> = ap_activity.cc.clone();
                let mut inboxes: Vec<String> = Vec::new();
                let mut notification_id: Option<i64> = None;

                to.retain(|x| x != &format!("{}/followers", &ap_activity.actor));
                cc.retain(|x| x != &format!("{}/followers", &ap_activity.actor));

                if to.contains(&"https://www.w3.org/ns/activitystreams#Public".to_string()) {
                    cc.push(format!("{}/followers", serialized_actor.actor_uri));
                    inboxes = follower_inboxes(pooled_connection, &serialized_actor.followers);
                } else if cc.contains(&"https://www.w3.org/ns/activitystreams#Public".to_string()) {
                    to.push(format!("{}/followers", serialized_actor.actor_uri));
                    inboxes = follower_inboxes(pooled_connection, &serialized_actor.followers);
                }

                match get_actor_by_uri(pooled_connection, &ap_activity.actor) {
                    Ok(receipient) => {
                        if !receipient.local {
                            inboxes.push(receipient.inbox.unwrap());
                        } else {
                            notification_id = Some(receipient.id);
                        }

                        to.push(receipient.actor_uri);
                    }
                    Err(_) => panic!(
                        "Error: Actor '{}' should exist in order to create a reaction!",
                        &ap_activity.actor
                    ),
                }

                to.dedup();
                cc.dedup();
                inboxes.dedup();

                let reaction_activity = activitypub::Activity::new(
                    _type,
                    &serialized_actor.actor_uri,
                    serde_json::json!(object_id),
                    to,
                    cc,
                );
                let activity = insert_activity(pooled_connection, reaction_activity.clone().into());

                federator::enqueue(
                    serialized_actor,
                    serde_json::json!(reaction_activity),
                    inboxes,
                );

                if notification_id.is_some() {
                    notification::insert(
                        pooled_connection,
                        Notification::new(activity.id, notification_id.unwrap()),
                    );
                }
            }
            Err(_) => (),
        }
    }
}

pub fn public_activities(pooled_connection: &PooledConnection) -> JsonValue {
    match timeline::public_activities(pooled_connection) {
        Ok(activities) => mastodon_api::controller::cached_statuses(pooled_connection, activities),
        Err(_) => json!({"error": "An error occured while querying public activities"}),
    }
}

pub fn status(
    pooled_connection: &PooledConnection,
    actor: &str,
    mut content: &str,
    visibility: &str,
    in_reply_to: Option<i64>,
) -> i64 {
    let serialized_actor: Actor = get_actor_by_uri(pooled_connection, &actor).unwrap();

    let mut to: Vec<String> = Vec::new();
    let mut cc: Vec<String> = Vec::new();
    let mut inboxes: Vec<String> = Vec::new();
    let mut notifications: Vec<i64> = Vec::new();
    let mut tags: Vec<serde_json::Value> = Vec::new();
    let mut in_reply_to_id: Option<String>;

    let mut parsed_content = content.to_string();

    let acct_regex = Regex::new(r"@[a-zA-Z0-9._-]+(@[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+\w)?").unwrap();

    for capture in acct_regex.captures_iter(content) {
        match get_actor_by_acct(
            pooled_connection,
            &capture
                    .get(0)
                    .unwrap()
                    .as_str()
                    .to_string()
                    .split_off(1),
        ) {
            Ok(mention) => {
                let acct = capture.get(0).unwrap().as_str();
                let tag: Tag = Tag {
                    _type: String::from("Mention"),
                    href: mention.actor_uri.clone(),
                    name: acct.to_string(),
                };
                tags.push(serde_json::to_value(tag).unwrap());
                to.push(mention.actor_uri.clone());

                if !mention.local {
                    inboxes.push(mention.inbox.unwrap());
                } else {
                    notifications.push(mention.id);
                }

                parsed_content = str::replace(
                    &parsed_content,
                    &acct,
                    &format!(
                        "<a class=\"mention\" href=\"{uri}\">{acct}</a>",
                        uri = mention.actor_uri,
                        acct = format!("@{}", mention.preferred_username)
                    ),
                );
            }
            Err(_) => (),
        }
    }

    let in_reply_to_id = match in_reply_to {
        Some(id) => match get_activity_by_id(pooled_connection, id) {
            Ok(activity) => {
                let object: activitypub::Object =
                    serde_json::from_value(activity.data["object"].clone()).unwrap();
                let actor: Result<Actor, diesel::result::Error> =
                    get_actor_by_uri(pooled_connection, &object.attributedTo);

                match actor {
                    Ok(actor) => {
                        if !actor.local {
                            inboxes.push(actor.inbox.unwrap());
                        } else {
                            notifications.push(actor.id);
                        }
                        to.push(actor.actor_uri);
                    }
                    Err(_) => (),
                }

                Some(object.id)
            }
            Err(_) => None,
        },
        None => None,
    };

    match visibility {
        "public" => {
            to.push("https://www.w3.org/ns/activitystreams#Public".to_string());
            cc.push(format!("{}/followers", actor));
            inboxes.extend(follower_inboxes(
                pooled_connection,
                &serialized_actor.followers,
            ));
        }
        "unlisted" => {
            to.push(format!("{}/followers", actor));
            cc.push("https://www.w3.org/ns/activitystreams#Public".to_string());
            inboxes.extend(follower_inboxes(
                pooled_connection,
                &serialized_actor.followers,
            ));
        }
        "private" => {
            to.push(format!("{}/followers", actor));
            inboxes.extend(follower_inboxes(
                pooled_connection,
                &serialized_actor.followers,
            ));
        }
        _ => (),
    }

    to.dedup();
    cc.dedup();
    inboxes.dedup();
    notifications.dedup();

    let note = serde_json::json!(activitypub::Object::note(
        &actor,
        in_reply_to_id,
        &parsed_content,
        to.clone(),
        cc.clone(),
        tags,
    ));
    let create_activity = activitypub::Activity::new("Create", &actor, note, to, cc);
    let activity = insert_activity(pooled_connection, create_activity.clone().into());

    // Move the delivery into a different thread, as it might take a while to complete
    // and therefore might delay the API response significantly.
    thread::spawn(move || {
        federator::enqueue(
            serialized_actor,
            serde_json::json!(create_activity),
            inboxes,
        );
    });

    // Generate notifications for local actors
    for notification in notifications {
        notification::insert(
            pooled_connection,
            Notification::new(activity.id, notification),
        );
    }

    return activity.id;
}

pub fn unfollow(pooled_connection: &PooledConnection, sender: &str, receipient: &str) {
    let serialized_actor: Actor = get_actor_by_uri(pooled_connection, sender).unwrap();

    match get_actor_by_uri(pooled_connection, receipient) {
        Ok(followee) => {
            if is_actor_followed_by(pooled_connection, &followee.actor_uri, sender).unwrap() {
                let activitypub_followers: Vec<serde_json::Value> =
                    serde_json::from_value(followee.followers["activitypub"].to_owned()).unwrap();
                let index = activitypub_followers
                    .iter()
                    .position(|ref follow| follow["href"].as_str().unwrap() == sender)
                    .unwrap();
                let follow_id: String =
                    serde_json::from_value(activitypub_followers[index]["activity_id"].to_owned())
                        .unwrap();

                // TODO: This should fake a `Follow` activity, if the original activity was lost
                let undo_activity = activitypub::Activity::new(
                    "Undo",
                    sender,
                    get_ap_activity_by_id(pooled_connection, &follow_id)
                        .unwrap()
                        .data,
                    vec![followee.actor_uri],
                    vec![],
                );
                let activity = insert_activity(pooled_connection, undo_activity.clone().into());
                remove_follow(receipient, sender);

                if !followee.local {
                    federator::enqueue(
                        serialized_actor,
                        serde_json::json!(&undo_activity),
                        vec![followee.inbox.unwrap()],
                    );
                } else {
                    notification::insert(
                        pooled_connection,
                        Notification::new(activity.id, followee.id),
                    );
                }
            }
        }
        Err(_) => (),
    }
}

fn follower_inboxes(db_connection: &PgConnection, followers: &serde_json::Value) -> Vec<String> {
    let ap_followers = serde_json::from_value(followers["activitypub"].clone());
    let follow_data: Vec<serde_json::Value> = ap_followers.unwrap();
    let mut inboxes: Vec<String> = vec![];

    for follower in follow_data {
        match get_actor_by_uri(db_connection, follower["href"].as_str().unwrap()) {
            Ok(actor) => {
                if !actor.local {
                    inboxes.push(actor.inbox.unwrap());
                }
            }
            Err(_) => (),
        }
    }
    return inboxes;
}
