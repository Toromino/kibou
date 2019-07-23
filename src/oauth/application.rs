use database::models::QueryOAuthApplication;
use database::schema::oauth_applications;
use database::schema::oauth_applications::dsl::*;
use diesel::pg::PgConnection;
use diesel::query_dsl::RunQueryDsl;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use openssl::bn::BigNum;
use openssl::bn::MsbOption;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Application {
    pub id: i64,
    pub client_name: Option<String>,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: String,
    pub scopes: String,
    pub website: Option<String>,
}

pub fn verify_credentials(
    db_connection: &PgConnection,
    application_id: String,
    application_secret: String,
) -> bool {
    match oauth_applications
        .filter(client_id.eq(application_id))
        .filter(client_secret.eq(application_secret))
        .limit(1)
        .first::<QueryOAuthApplication>(db_connection)
    {
        Ok(_application) => true,
        Err(_e) => false,
    }
}

pub fn get_application_by_client_id(
    db_connection: &PgConnection,
    application_id: String,
) -> Result<Application, diesel::result::Error> {
    match oauth_applications
        .filter(client_id.eq(application_id))
        .limit(1)
        .first::<QueryOAuthApplication>(db_connection)
    {
        Ok(application) => Ok(serialize_application(application)),
        Err(e) => Err(e),
    }
}

pub fn create(db_connection: &PgConnection, mut app: Application) -> Application {
    let mut big_num: BigNum = BigNum::new().unwrap();
    big_num
        .rand(256, MsbOption::MAYBE_ZERO, true)
        .expect("Error generating client secret");
    app.client_id = Uuid::new_v4().to_string();
    app.client_secret = big_num.to_hex_str().unwrap().to_string();

    insert(&db_connection, &app);

    // *Note*
    // Returning application from the database, as the id of 'app'
    // is not updated yet.
    return get_application_by_client_id(&db_connection, app.client_id).unwrap();
}

fn serialize_application(sql_application: QueryOAuthApplication) -> Application {
    Application {
        id: sql_application.id,
        client_name: sql_application.client_name,
        client_id: sql_application.client_id,
        client_secret: sql_application.client_secret,
        redirect_uris: sql_application.redirect_uris,
        scopes: sql_application.scopes,
        website: sql_application.website,
    }
}

fn insert(db_connection: &PgConnection, app: &Application) {
    let new_app = (
        client_name.eq(&app.client_name),
        client_id.eq(&app.client_id),
        client_secret.eq(&app.client_secret),
        redirect_uris.eq(&app.redirect_uris),
        scopes.eq(&app.scopes),
        website.eq(&app.website),
    );

    diesel::insert_into(oauth_applications::table)
        .values(new_app)
        .execute(db_connection)
        .expect("Error creating oauth application");
}
