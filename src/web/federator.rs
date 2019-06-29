use actor::Actor;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::DATE;
use reqwest::header::HOST;
use serde_json;
use url::Url;
use web::http_signatures::Signature;

pub fn enqueue(mut actor: Actor, activity: serde_json::Value, inboxes: Vec<String>) {
    let client = reqwest::Client::new();

    for inbox in inboxes {
        let url = Url::parse(&inbox).unwrap();
        let host = url.host_str().unwrap();
        let mut signature =
            Signature::new(&format!("{}#main-key", &actor.actor_uri), url.path(), host);
        signature.sign(&mut actor);

        println!("Federating activity to inbox: {}", inbox);

        let request = client
            .post(&inbox)
            .header(CONTENT_TYPE, "application/ld+json; profile=\"https://www.w3.org/ns/activitystreams\"")
            .header(DATE, chrono::Utc::now().to_rfc2822().to_string())
            .header(HOST, host)
            .header("Signature", signature.build_header())
            .json(&activity)
            .send();
    }
}
