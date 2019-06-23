use chrono::{Utc, NaiveDateTime};
use database::schema::notifications;
use diesel::{RunQueryDsl, PgConnection};

#[derive(Insertable, Queryable, PartialEq, QueryableByName, Clone)]
#[table_name = "notifications"]
pub struct Notification {
    id: i64,
    activity_id: i64,
    actor_id: i64,
    created: NaiveDateTime,
    modified: NaiveDateTime
}

impl Notification {
    fn new(activity_id: i64, actor_id: i64) -> Notification {
        Notification {
            id: 0,
            activity_id: activity_id,
            actor_id: actor_id,
            created: Utc::now().naive_utc(),
            modified: Utc::now().naive_utc()
        }
    }
}

pub fn insert(db_connection: &PgConnection, notification: Notification) {
    diesel::insert_into(notifications::table)
        .values(notification)
        .execute(db_connection)
        .expect("Error creating notification");
}

