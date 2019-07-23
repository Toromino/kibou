use actor::Actor;
use chrono::{NaiveDateTime, Utc};
use database::models::InsertNotification;
use database::schema::notifications;
use diesel::{sql_query, PgConnection, RunQueryDsl};

#[derive(Queryable, PartialEq, QueryableByName, Clone)]
#[table_name = "notifications"]
pub struct Notification {
    id: i64,
    activity_id: i64,
    actor_id: i64,
    created: NaiveDateTime,
    modified: NaiveDateTime,
}

impl Notification {
    pub fn new(activity_id: i64, actor_id: i64) -> Notification {
        Notification {
            id: 0,
            activity_id: activity_id,
            actor_id: actor_id,
            created: Utc::now().naive_utc(),
            modified: Utc::now().naive_utc(),
        }
    }
}

pub fn notifications_for_actor(
    db_connection: &PgConnection,
    actor: &Actor,
    max_id: Option<i64>,
    since_id: Option<i64>,
    min_id: Option<i64>,
    limit: Option<i64>,
) -> Result<Vec<i64>, diesel::result::Error> {
    let id = if max_id.is_some() {
        format!("AND id < {} ORDER BY id DESC", &max_id.unwrap().to_string())
    } else if since_id.is_some() {
        format!(
            "AND id > {} ORDER BY id DESC",
            &since_id.unwrap().to_string()
        )
    } else if min_id.is_some() {
        format!("AND id > {} ORDER BY id ASC", &min_id.unwrap().to_string())
    } else {
        String::from("ORDER BY id DESC")
    };

    match sql_query(format!(
        "SELECT * \
         FROM notifications \
         WHERE \
         actor_id = '{actor_id}' \
         {order} \
         LIMIT {limit};",
        actor_id = actor.id,
        order = id,
        limit = limit.unwrap_or_else(|| 20).to_string()
    ))
    .load::<Notification>(db_connection)
    {
        Ok(notifications) => Ok(notifications
            .iter()
            .map(|notification| notification.activity_id)
            .collect()),
        Err(e) => Err(e),
    }
}

pub fn insert(db_connection: &PgConnection, notification: Notification) {
    diesel::insert_into(notifications::table)
        .values(InsertNotification {
            activity_id: notification.activity_id,
            actor_id: notification.actor_id,
            created: notification.created,
            modified: notification.modified,
        })
        .execute(db_connection)
        .expect("Error creating notification");
}
