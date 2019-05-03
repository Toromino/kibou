//! Kibou_API provides a common application layer between data reprentations of modules such as
//! ActivityPub, Mastodon_API and internal reprentations.
//!
//! Furthermore Kibou_API will implement endpoints which are unique to the Kibou backend.
//!

use activity::{get_activity_by_id, get_ap_activity_by_id};
use activitypub::activity::Tag;
use activitypub::actor::{add_follow, remove_follow};
use activitypub::controller as ap_controller;
use actor::{get_actor_by_acct, get_actor_by_uri, is_actor_followed_by, Actor};
use database;
use diesel::PgConnection;
use html;
use regex::Regex;
use web_handler::federator;

pub fn follow(actor: String, object: String) {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &actor).unwrap();

    match get_actor_by_uri(&database, &object) {
        Ok(followee) => {
            if !is_actor_followed_by(&database, &followee, &actor).unwrap() {
                let activitypub_activity_follow = ap_controller::follow(&actor, &object);

                if !followee.local {
                    federator::enqueue(
                        serialized_actor,
                        serde_json::json!(&activitypub_activity_follow),
                        vec![followee.inbox.unwrap()],
                    );
                } else {
                    add_follow(&object, &actor, &activitypub_activity_follow.id);
                }
            }
        }
        Err(_) => (),
    }
}

pub fn status_build(
    actor: String,
    mut content: String,
    visibility: &str,
    in_reply_to: Option<String>,
) -> String {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &actor).unwrap();

    let mut direct_receipients: Vec<String> = Vec::new();
    let mut receipients: Vec<String> = Vec::new();
    let mut inboxes: Vec<String> = Vec::new();
    let mut tags: Vec<serde_json::Value> = Vec::new();

    let parsed_mentions = parse_mentions(html::strip_tags(content));
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

    inboxes.dedup();

    let activitypub_note = ap_controller::note(
        &actor,
        handle_in_reply_to(in_reply_to),
        content,
        direct_receipients.clone(),
        receipients.clone(),
        tags,
    );
    let activitypub_activity_create = ap_controller::create(
        &actor,
        serde_json::to_value(activitypub_note).unwrap(),
        direct_receipients,
        receipients,
    );
    federator::enqueue(
        serialized_actor,
        serde_json::json!(&activitypub_activity_create),
        inboxes,
    );

    return activitypub_activity_create.id;
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

fn handle_in_reply_to(local_id: Option<String>) -> Option<String> {
    let database = database::establish_connection();

    if local_id.is_some() {
        match get_activity_by_id(&database, local_id.unwrap().parse::<i64>().unwrap()) {
            Ok(activity) => Some(activity.data["object"]["id"].as_str().unwrap().to_string()),
            Err(_) => None,
        }
    } else {
        return None;
    }
}

fn parse_mentions(content: String) -> (Vec<String>, Vec<String>, Vec<serde_json::Value>, String) {
    let acct_regex = Regex::new(r"@[a-zA-Z0-9._-]+@[a-zA-Z0-9._-]+\.[a-zA-Z0-9_-]+\w").unwrap();
    let database = database::establish_connection();

    let mut receipients: Vec<String> = vec![];
    let mut inboxes: Vec<String> = vec![];
    let mut new_content: String = content.clone();
    let mut tags: Vec<serde_json::Value> = vec![];

    for mention in acct_regex.captures_iter(&content) {
        match get_actor_by_acct(
            &database,
            mention.get(0).unwrap().as_str().to_string().split_off(1),
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
                        acct = mention.get(0).unwrap().as_str()
                    ),
                );
            }
            Err(_) => (),
        }
    }
    (receipients, inboxes, tags, new_content)
}
