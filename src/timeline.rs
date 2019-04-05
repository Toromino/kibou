use activity::serialize_activity;
use activity::Activity;
use actor;
use actor::Actor;
use database::models::QueryActivity;
use diesel::pg::PgConnection;
use diesel::query_dsl::RunQueryDsl;
use diesel::sql_query;
use env;

pub fn get_home_timeline(
    db_connection: &PgConnection,
    actor: Actor,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Activity>, diesel::result::Error> {
    let query_limit = if limit.is_some() { limit.unwrap() } else { 20 };

    match sql_query(format!(
        "SELECT * \
         FROM activities \
         WHERE \
         (data->>'type' = 'Create' OR \
         data->>'type' = 'Announce') AND \
         (actor_uri = ANY (ARRAY['{followees}']::varchar(255)[]) OR \
         actor_uri = '{actor_uri}') \
         {id} \
         LIMIT {limit};",
        followees = actor::get_actor_followees(db_connection, &actor.actor_uri)
            .unwrap()
            .join("','"),
        actor_uri = actor.actor_uri,
        id = get_id_order_query(max_id, since_id, min_id),
        limit = query_limit
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity) => {
            let mut serialized_activities: Vec<Activity> = vec![];

            for object in activity {
                serialized_activities.push(serialize_activity(object));
            }

            Ok(serialized_activities)
        }
        Err(e) => Err(e),
    }
}

pub fn get_public_timeline(
    db_connection: &PgConnection,
    local: bool,
    only_media: bool,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<Activity>, diesel::result::Error> {
    let query_local = if local {
        format!(
            "AND data->>'actor' LIKE '{base_scheme}://{base_domain}/%'",
            base_scheme = env::get_value(String::from("endpoint.base_scheme")),
            base_domain = env::get_value(String::from("endpoint.base_domain"))
        )
    } else {
        String::from("")
    };

    let query_limit = if limit.is_some() { limit.unwrap() } else { 20 };

    match sql_query(format!(
        "SELECT * \
         FROM activities \
         WHERE data->>'type' = 'Create' \
         AND (data->>'to')::jsonb ? 'https://www.w3.org/ns/activitystreams#Public' \
         {local} \
         {id} \
         LIMIT {limit};",
        local = query_local,
        id = get_id_order_query(max_id, since_id, min_id),
        limit = query_limit
    ))
    .load::<QueryActivity>(db_connection)
    {
        Ok(activity) => {
            let mut serialized_activities: Vec<Activity> = vec![];

            for object in activity {
                serialized_activities.push(serialize_activity(object));
            }

            Ok(serialized_activities)
        }
        Err(e) => Err(e),
    }
}

fn get_id_order_query(max_id: Option<i64>, since_id: Option<i64>, min_id: Option<i64>) -> String {
    if max_id.is_some() {
        format!("AND id < {} ORDER BY id DESC", max_id.unwrap())
    } else if since_id.is_some() {
        format!("AND id > {} ORDER BY id DESC", since_id.unwrap())
    } else if min_id.is_some() {
        format!("AND id > {} ORDER BY id ASC", since_id.unwrap())
    } else {
        String::from("ORDER BY id DESC")
    }
}
