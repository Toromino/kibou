use activitypub::activity::Tag;
use activitypub::controller::activity_create;
use activitypub::controller::note;
use actor::get_actor_by_acct;
use actor::get_actor_by_uri;
use actor::Actor;
use database;
use regex::Regex;
use web_handler::federator;

pub fn build(actor: String, mut content: String, visiblity: &str, in_reply_to: Option<String>) {
    let database = database::establish_connection();
    let serialized_actor: Actor = get_actor_by_uri(&database, &actor).unwrap();

    let mut direct_receipients: Vec<String> = vec![];
    let mut receipients: Vec<String> = vec![];
    let mut inboxes: Vec<String> = vec![];
    let mut tags: Vec<serde_json::Value> = vec![];

    let parsed_mentions = parse_mentions(content.clone());
    direct_receipients.extend(parsed_mentions.0);
    inboxes.extend(parsed_mentions.1);
    tags.extend(parsed_mentions.2);
    content = parsed_mentions.3;

    match visiblity {
        "public" => {
            direct_receipients.push("https://www.w3.org/ns/activitystreams#Public".to_string());
            receipients.push(format!("{}/followers", actor));
        }

        "unlisted" => {
            direct_receipients.push(format!("{}/followers", actor));
            receipients.push("https://www.w3.org/ns/activitystreams#Public".to_string());
        }

        "private" => {
            direct_receipients.push(format!("{}/followers", actor));
        }

        _ => (),
    }

    let activitypub_note = note(
        &actor,
        in_reply_to,
        content,
        direct_receipients.clone(),
        receipients.clone(),
        tags,
    );
    let activitypub_activity_create = activity_create(
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
