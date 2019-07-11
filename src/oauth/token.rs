use chrono::prelude::*;
use chrono::Duration;
use chrono::NaiveDateTime;
use database;
use database::models::QueryOauthToken;
use database::schema::oauth_tokens;
use database::schema::oauth_tokens::dsl::*;
use diesel::pg::PgConnection;
use diesel::query_dsl::RunQueryDsl;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use oauth::application::verify_credentials;
use oauth::authorization::get_authorization_by_code;
use openssl::bn::BigNum;
use openssl::bn::MsbOption;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub actor: String,
    pub valid_until: String,
    pub scope: String,
}

#[derive(Debug, FromForm)]
pub struct TokenForm {
    pub client_id: String,
    pub client_secret: String,
    pub grant_type: String,
    pub code: String,
    pub redirect_uri: String,
}

fn serialize_token(sql_token: QueryOauthToken) -> Token {
    Token {
        access_token: sql_token.access_token,
        refresh_token: sql_token.refresh_token,
        token_type: String::from("Bearer"),
        actor: sql_token.actor,
        valid_until: sql_token.valid_until.to_string(),
        scope: String::from("read+write+follow"),
    }
}

pub fn verify_token(
    db_connection: &PgConnection,
    _token: String,
) -> Result<Token, diesel::result::Error> {
    match oauth_tokens
        .filter(access_token.eq(_token))
        .limit(1)
        .first::<QueryOauthToken>(db_connection)
    {
        Ok(sql_token) => Ok(serialize_token(sql_token)),
        Err(e) => Err(e),
    }
}

pub fn get_token(form: TokenForm) -> JsonValue {
    let db_connection = database::establish_connection();

    if verify_credentials(&db_connection, form.client_id, form.client_secret) {
        match get_authorization_by_code(&db_connection, form.code) {
            Ok(authorization) => json!(create(&authorization.actor)),
            Err(_) => json!({"Error": "OAuth authorization code is invalid."}),
        }
    } else {
        json!({"Error": "OAuth application credentials are invalid."})
    }
}

pub fn create(actor_username: &str) -> Token {
    let db_connection = database::establish_connection();
    let mut access_token_num: BigNum = BigNum::new().unwrap();
    let mut refresh_token_num: BigNum = BigNum::new().unwrap();
    let utc_time: chrono::DateTime<Utc> = Utc::now();
    let expiration_date: chrono::DateTime<Utc> = utc_time + Duration::days(30);

    access_token_num
        .rand(256, MsbOption::MAYBE_ZERO, true)
        .expect("Error generating access token");
    refresh_token_num
        .rand(256, MsbOption::MAYBE_ZERO, true)
        .expect("Error generating refresh token");

    let new_token: Token = Token {
        access_token: access_token_num.to_hex_str().unwrap().to_string(),
        refresh_token: refresh_token_num.to_hex_str().unwrap().to_string(),
        actor: actor_username.to_string(),
        token_type: String::from("Bearer"),
        valid_until: expiration_date.timestamp().to_string(),
        scope: String::from("read write follow"),
    };

    insert(&db_connection, &new_token);
    return new_token;
}

fn insert(db_connection: &PgConnection, mut _token: &Token) {
    let parsed_expiration_date: NaiveDateTime =
        chrono::NaiveDateTime::from_timestamp(_token.valid_until.parse::<i64>().unwrap(), 0);

    let new_token = (
        access_token.eq(&_token.access_token),
        actor.eq(&_token.actor),
        refresh_token.eq(&_token.refresh_token),
        valid_until.eq(&parsed_expiration_date),
    );

    diesel::insert_into(oauth_tokens::table)
        .values(new_token)
        .execute(db_connection)
        .expect("Error creating oauth application");
}
