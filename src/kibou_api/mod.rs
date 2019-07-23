//! Kibou_API provides a common application layer between data reprentations of modules such as
//! ActivityPub, Mastodon_API and internal reprentations.
//!
//! Furthermore Kibou_API will implement endpoints which are unique to the Kibou backend.
//!

pub mod routes;

use activity::{
    get_activity_by_id, get_ap_activity_by_id, get_ap_object_by_id, type_exists_for_object_id,
};
use activitypub::activity::{serialize_from_internal_activity, Tag};
use activitypub::actor::{add_follow, remove_follow};
use activitypub::controller as ap_controller;
use actor::{get_actor_by_acct, get_actor_by_id, get_actor_by_uri, is_actor_followed_by, Actor};
use database;
use database::PooledConnection;
use diesel::PgConnection;
use html;
use mastodon_api;
use regex::Regex;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use std::thread;
use timeline;
use web::federator;

pub fn follow(sender: &str, receipient: &str) {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &sender).unwrap();

    if sender != receipient {
        match get_actor_by_uri(&database, &receipient) {
            Ok(followee) => {
                if !is_actor_followed_by(&database, &followee, &sender).unwrap() {
                    let activitypub_activity_follow = ap_controller::follow(sender, receipient);

                    if !followee.local {
                        federator::enqueue(
                            serialized_actor,
                            serde_json::json!(&activitypub_activity_follow),
                            vec![followee.inbox.unwrap()],
                        );
                    } else {
                        add_follow(receipient, sender, &activitypub_activity_follow.id);
                    }
                }
            }
            Err(_) => (),
        }
    }
}

pub fn react(actor: &i64, _type: &str, object_id: &str) {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_id(&database, actor).expect("Actor should exist!");

    if !type_exists_for_object_id(&database, _type, &serialized_actor.actor_uri, object_id)
        .unwrap_or_else(|_| true)
    {
        match get_ap_object_by_id(&database, object_id) {
            Ok(activity) => {
                let ap_activity = serialize_from_internal_activity(activity);

                let mut to: Vec<String> = ap_activity.to.clone();
                let mut cc: Vec<String> = ap_activity.cc.clone();
                let mut inboxes: Vec<String> = Vec::new();

                to.retain(|x| x != &format!("{}/followers", &ap_activity.actor));
                cc.retain(|x| x != &format!("{}/followers", &ap_activity.actor));

                if to.contains(&"https://www.w3.org/ns/activitystreams#Public".to_string()) {
                    cc.push(format!("{}/followers", serialized_actor.actor_uri));
                    inboxes = handle_follower_inboxes(&database, &serialized_actor.followers);
                } else if cc.contains(&"https://www.w3.org/ns/activitystreams#Public".to_string()) {
                    to.push(format!("{}/followers", serialized_actor.actor_uri));
                    inboxes = handle_follower_inboxes(&database, &serialized_actor.followers);
                }

                match get_actor_by_uri(&database, &ap_activity.actor) {
                    Ok(foreign_actor) => {
                        if !foreign_actor.local {
                            to.push(foreign_actor.actor_uri);
                            inboxes.push(foreign_actor.inbox.unwrap());
                        }
                    }
                    Err(_) => eprintln!(
                        "Error: Actor '{}' should exist in order to create a reaction!",
                        &ap_activity.actor
                    ),
                }

                to.dedup();
                cc.dedup();
                inboxes.dedup();

                match _type {
                    "Announce" => {
                        let new_activity =
                            ap_controller::announce(&serialized_actor.actor_uri, object_id, to, cc);
                        federator::enqueue(
                            serialized_actor,
                            serde_json::json!(&new_activity),
                            inboxes,
                        );
                    }
                    "Like" => {
                        let new_activity =
                            ap_controller::like(&serialized_actor.actor_uri, object_id, to, cc);
                        federator::enqueue(
                            serialized_actor,
                            serde_json::json!(&new_activity),
                            inboxes,
                        );
                    }
                    _ => (),
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

pub fn status_build(
    actor: String,
    mut content: String,
    visibility: &str,
    in_reply_to: Option<String>,
) -> i64 {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &actor).unwrap();

    let mut direct_receipients: Vec<String> = Vec::new();
    let mut receipients: Vec<String> = Vec::new();
    let mut inboxes: Vec<String> = Vec::new();
    let mut tags: Vec<serde_json::Value> = Vec::new();
    let in_reply_to_id: Option<String>;

    let parsed_mentions = parse_mentions(html::to_plain_text(&content));
    direct_receipients.extend(parsed_mentions.0);
    inboxes.extend(parsed_mentions.1);
    tags.extend(parsed_mentions.2);
    content = parsed_mentions.3;

    match visibility {
        "public" => {
            direct_receipients.push("https://www.w3.org/ns/activitystreams#Public".to_string());
            receipients.push(format!("{}/followers", actor));
            inboxes.extend(handle_follower_inboxes(
                &database,
                &serialized_actor.followers,
            ));
        }

        "unlisted" => {
            direct_receipients.push(format!("{}/followers", actor));
            receipients.push("https://www.w3.org/ns/activitystreams#Public".to_string());
            inboxes.extend(handle_follower_inboxes(
                &database,
                &serialized_actor.followers,
            ));
        }

        "private" => {
            direct_receipients.push(format!("{}/followers", actor));
            inboxes.extend(handle_follower_inboxes(
                &database,
                &serialized_actor.followers,
            ));
        }

        _ => (),
    }

    if in_reply_to.is_some() {
        match get_activity_by_id(&database, in_reply_to.unwrap().parse::<i64>().unwrap()) {
            Ok(activity) => {
                in_reply_to_id = Some(activity.data["object"]["id"].as_str().unwrap().to_string());

                match get_actor_by_uri(
                    &database,
                    activity.data["object"]["attributedTo"].as_str().unwrap(),
                ) {
                    Ok(actor) => {
                        direct_receipients.push(actor.actor_uri);
                        if !actor.local {
                            inboxes.push(actor.inbox.unwrap());
                        }
                    }
                    Err(_) => (),
                }
            }
            Err(_) => in_reply_to_id = None,
        }
    } else {
        in_reply_to_id = None;
    }

    direct_receipients.dedup();
    receipients.dedup();
    inboxes.dedup();

    let activitypub_note = ap_controller::note(
        &actor,
        in_reply_to_id,
        content,
        direct_receipients.clone(),
        receipients.clone(),
        tags,
    );
    let activitypub_activity_create = ap_controller::create(
        &actor,
        serde_json::to_value(&activitypub_note).unwrap(),
        direct_receipients,
        receipients,
    );
    thread::spawn(move || {
        federator::enqueue(
            serialized_actor,
            serde_json::json!(&activitypub_activity_create),
            inboxes,
        );
    });

    return get_ap_object_by_id(&database, &activitypub_note.id)
        .unwrap()
        .id;
}

pub fn unfollow(actor: String, object: String) {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &actor).unwrap();

    match get_actor_by_uri(&database, &object) {
        Ok(followee) => {
            if is_actor_followed_by(&database, &followee, &actor).unwrap() {
                let activitypub_followers: Vec<serde_json::Value> =
                    serde_json::from_value(followee.followers["activitypub"].to_owned()).unwrap();

                let activitypub_follow_index = activitypub_followers
                    .iter()
                    .position(|ref follow| follow["href"].as_str().unwrap() == &actor)
                    .unwrap();
                let activitypub_follow_id: String = serde_json::from_value(
                    activitypub_followers[activitypub_follow_index]["activity_id"].to_owned(),
                )
                .unwrap();

                let activitypub_activity_unfollow = ap_controller::undo(
                    &actor,
                    get_ap_activity_by_id(&database, &activitypub_follow_id)
                        .unwrap()
                        .data,
                    vec![followee.actor_uri],
                    vec![],
                );
                remove_follow(&object, &actor);

                if !followee.local {
                    federator::enqueue(
                        serialized_actor,
                        serde_json::json!(&activitypub_activity_unfollow),
                        vec![followee.inbox.unwrap()],
                    );
                }
            }
        }
        Err(_) => (),
    }
}

fn handle_follower_inboxes(
    db_connection: &PgConnection,
    followers: &serde_json::Value,
) -> Vec<String> {
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

fn parse_mentions(content: String) -> (Vec<String>, Vec<String>, Vec<serde_json::Value>, String) {
    let acct_regex = Regex::new(r"@[a-zA-Z0-9._-]+(@[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+\w)?").unwrap();
    let database = database::establish_connection();

    let mut receipients: Vec<String> = vec![];
    let mut inboxes: Vec<String> = vec![];
    let mut new_content: String = content.clone();
    let mut tags: Vec<serde_json::Value> = vec![];

    for mention in acct_regex.captures_iter(&content) {
        match get_actor_by_acct(
            &database,
            &mention.get(0).unwrap().as_str().to_string().split_off(1),
        ) {
            Ok(actor) => {
                let tag: Tag = Tag {
                    _type: String::from("Mention"),
                    href: actor.actor_uri.clone(),
                    name: mention.get(0).unwrap().as_str().to_string(),
                };

                if !actor.local {
                    inboxes.push(actor.inbox.unwrap());
                }
                receipients.push(actor.actor_uri.clone());
                tags.push(serde_json::to_value(tag).unwrap());
                new_content = str::replace(
                    &new_content,
                    mention.get(0).unwrap().as_str(),
                    &format!(
                        "<a class=\"mention\" href=\"{uri}\">{acct}</a>",
                        uri = actor.actor_uri,
                        acct = format!("@{}", actor.preferred_username)
                    ),
                );
            }
            Err(_) => (),
        }
    }
    (receipients, inboxes, tags, new_content)
}
