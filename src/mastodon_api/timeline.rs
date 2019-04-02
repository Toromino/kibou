use mastodon_api::status::serialize as serialize_status;
use mastodon_api::status::Status;
use oauth::token::verify_token;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use timeline::{get_home_timeline, get_public_timeline};
use {actor, database};

#[derive(FromForm)]
pub struct HomeTimeline {
    pub max_id: Option<i64>,
    pub since_id: Option<i64>,
    pub min_id: Option<i64>,
    pub limit: Option<i64>,
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

pub fn get_home_timeline_json(parameters: HomeTimeline, token: String) -> JsonValue {
    match home_timeline(parameters, token) {
        Ok(statuses) => json!(statuses),
        Err(_) => json!({"error": "An error occured while generating timeline."}),
    }
}

pub fn get_public_timeline_json(parameters: PublicTimeline) -> JsonValue {
    match public_timeline(parameters) {
        Ok(statuses) => json!(statuses),
        Err(_) => json!({"error": "An error occured while generating timeline."}),
    }
}

fn home_timeline(
    parameters: HomeTimeline,
    token: String,
) -> Result<Vec<Status>, diesel::result::Error> {
    let database = database::establish_connection();

    match verify_token(&database, token) {
        Ok(token) => match actor::get_local_actor_by_preferred_username(&database, token.actor) {
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
                            if let Ok(valid_status) = serialize_status(status) {
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
                if let Ok(valid_status) = serialize_status(status) {
                    serialized_statuses.push(valid_status)
                }
            }

            Ok(serialized_statuses)
        }
        Err(e) => Err(e),
    }
}
