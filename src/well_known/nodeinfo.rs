use activity::count_local_ap_notes;
use actor::count_local_actors;
use database;
use env;
use rocket_contrib::json;
use rocket_contrib::json::JsonValue;

#[get("/.well-known/nodeinfo")]
pub fn nodeinfo() -> JsonValue {
    json!({
           "links": [
           {
               "rel": "http://nodeinfo.diaspora.software/ns/schema/2.0",
               "href": format!("{base_scheme}://{base_domain}/nodeinfo/2.0.json",
               base_scheme = env::get_value(String::from("endpoint.base_scheme")),
               base_domain = env::get_value(String::from("endpoint.base_domain")))
           },
           {
               "rel": "http://nodeinfo.diaspora.software/ns/schema/2.1",
               "href": format!("{base_scheme}://{base_domain}/nodeinfo/2.1.json",
               base_scheme = env::get_value(String::from("endpoint.base_scheme")),
               base_domain = env::get_value(String::from("endpoint.base_domain")))
           }]
    })
}

// NoteInfo protocol version 2.0 according to the schema at
// http://nodeinfo.diaspora.software/ns/schema/2.0
#[get("/nodeinfo/2.0.json")]
pub fn nodeinfo_v2() -> JsonValue {
    json!({
        "version": "2.0",
        "software": {
            "version": format!("{version}-testing",
                version = env!("CARGO_PKG_VERSION")),
            "name": env!("CARGO_PKG_NAME")
        },
        "protocols": [
        "activitypub"
        ],
        "nodeName": env::get_value(String::from("node.name")),
        "nodeDescription": env::get_value(String::from("node.description")),
        "services":{
            "outbound": [

            ],
            "inbound": [

            ]
        },
        "openRegistrations": get_open_registrations(),
        "usage": {
            "users": {
                "total": get_total_users()
            },
            "localPosts": get_local_posts()
        },
        "metadata": {

        },
        "features": [
            "mastodon_api",
            "webfinger"
        ]
    })
}

// NoteInfo protocol version 2.1 according to the schema at
// http://nodeinfo.diaspora.software/ns/schema/2.1
#[get("/nodeinfo/2.1.json")]
pub fn nodeinfo_v2_1() -> JsonValue {
    json!({
        "version": "2.1",
        "software": {
            "version": format!("{version}-testing",
                version = env!("CARGO_PKG_VERSION")),
            "name": env!("CARGO_PKG_NAME"),
            "repository": "https://git.cybre.club/kibouproject/kibou"
        },
        "protocols": [
        "activitypub"
        ],
        "nodeName": env::get_value(String::from("node.name")),
        "nodeDescription": env::get_value(String::from("node.description")),
        "services":{
            "outbound": [

            ],
            "inbound": [

            ]
        },
        "openRegistrations": get_open_registrations(),
        "usage": {
            "users": {
                "total": get_total_users()
            },
            "localPosts": get_local_posts()
        },
        "metadata": {

        },
        "features": [
            "mastodon_api",
            "webfinger"
        ]
    })
}

fn get_local_posts() -> usize {
    let database = database::establish_connection();
    count_local_ap_notes(&database).unwrap_or_else(|_| 0)
}

fn get_open_registrations() -> bool {
    env::get_value(String::from("node.registrations_enabled"))
        .parse::<bool>()
        .unwrap_or_else(|_| false)
}

fn get_total_users() -> usize {
    let database = database::establish_connection();
    count_local_actors(&database).unwrap_or_else(|_| 0)
}
