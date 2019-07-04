use actor;
use actor::Actor;
use database::models::QueryActivityId;
use database::runtime_escape;
use diesel::pg::PgConnection;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_query;
use env;

pub fn home_timeline(
    db_connection: &PgConnection,
    actor: Actor,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<i64>, diesel::result::Error> {
    let limit = match limit {
        Some(value) => value,
        None => 20,
    };

    let followees: Vec<String> = actor::get_actor_followees(db_connection, &actor.actor_uri)
        .unwrap()
        .iter()
        .map(|followee| followee.actor_uri.to_owned())
        .collect();

    match sql_query(format!(
        "SELECT id \
         FROM activities \
         WHERE \
         (data @> '{{\"type\": \"Create\"}}' OR \
         data @> '{{\"type\": \"Announce\"}}') AND \
         (actor_uri = ANY (ARRAY['{followees}']::varchar(255)[]) OR \
         actor_uri = '{actor_uri}') \
         {id} \
         LIMIT {limit};",
        followees = followees.join("','"),
        actor_uri = actor.actor_uri,
        id = prepare_order_query(max_id, since_id, min_id),
        limit = runtime_escape(&limit.to_string())
    ))
    .load::<QueryActivityId>(db_connection)
    {
        Ok(activities) => Ok(activities.iter().map(|activity| activity.id).collect()),
        Err(e) => Err(e),
    }
}

pub fn public_activities(db_connection: &PgConnection) -> Result<Vec<i64>, diesel::result::Error> {
    match sql_query(format!(
        "SELECT id \
         FROM activities \
         WHERE data @> '{{\"type\": \"Create\"}}' OR \
         data @> '{{\"type\": \"Announce\"}}' AND \
         data -> 'to' ? 'https://www.w3.org/ns/activitystreams#Public' \
         AND data->>'actor' LIKE '{base_scheme}://{base_domain}/%' \
         LIMIT 20;",
        base_scheme = env::get_value(String::from("endpoint.base_scheme")),
        base_domain = env::get_value(String::from("endpoint.base_domain"))
    ))
    .load::<QueryActivityId>(db_connection)
    {
        Ok(activities) => Ok(activities.iter().map(|activity| activity.id).collect()),
        Err(e) => Err(e),
    }
}

pub fn public_timeline(
    db_connection: &PgConnection,
    local: bool,
    _only_media: bool,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<i64>, diesel::result::Error> {
    let local = match local {
        true => format!(
            "AND data->>'actor' LIKE '{base_scheme}://{base_domain}/%'",
            base_scheme = env::get_value(String::from("endpoint.base_scheme")),
            base_domain = env::get_value(String::from("endpoint.base_domain"))
        ),
        false => String::from(""),
    };

    let limit = match limit {
        Some(value) => value,
        None => 20,
    };

    match sql_query(format!(
        "SELECT id \
         FROM activities \
         WHERE data @> '{{\"type\": \"Create\"}}' AND \
         data -> 'to' ? 'https://www.w3.org/ns/activitystreams#Public' \
         {local} \
         {id} \
         LIMIT {limit};",
        local = local,
        id = prepare_order_query(max_id, since_id, min_id),
        limit = runtime_escape(&limit.to_string())
    ))
    .load::<QueryActivityId>(db_connection)
    {
        Ok(activities) => Ok(activities.iter().map(|activity| activity.id).collect()),
        Err(e) => Err(e),
    }
}

pub fn user_timeline(
    db_connection: &PgConnection,
    actor: Actor,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<i64>, diesel::result::Error> {
    let limit = match limit {
        Some(value) => value,
        None => 20,
    };

    match sql_query(format!(
        "SELECT id \
         FROM activities \
         WHERE \
         (data @> '{{\"type\": \"Create\"}}' OR \
         data @> '{{\"type\": \"Announce\"}}') AND \
         actor_uri = '{actor_uri}' AND \
         (data -> 'to' ? 'https://www.w3.org/ns/activitystreams#Public' OR \
         data -> 'cc' ? 'https://www.w3.org/ns/activitystreams#Public') \
         {id} \
         LIMIT {limit};",
        actor_uri = runtime_escape(&actor.actor_uri),
        id = prepare_order_query(max_id, since_id, min_id),
        limit = runtime_escape(&limit.to_string())
    ))
    .load::<QueryActivityId>(db_connection)
    {
        Ok(activities) => Ok(activities.iter().map(|activity| activity.id).collect()),
        Err(e) => Err(e),
    }
}

fn prepare_order_query(max_id: Option<i64>, since_id: Option<i64>, min_id: Option<i64>) -> String {
    if max_id.is_some() {
        format!(
            "AND id < {} ORDER BY id DESC",
            runtime_escape(&max_id.unwrap().to_string())
        )
    } else if since_id.is_some() {
        format!(
            "AND id > {} ORDER BY id DESC",
            runtime_escape(&since_id.unwrap().to_string())
        )
    } else if min_id.is_some() {
        format!(
            "AND id > {} ORDER BY id ASC",
            runtime_escape(&min_id.unwrap().to_string())
        )
    } else {
        String::from("ORDER BY id DESC")
    }
}
