use actor::Actor;
use reqwest::header::DATE;
use reqwest::header::HOST;
use serde_json;
use url::Url;
use web_handler::http_signatures;

pub fn enqueue(mut actor: Actor, activity: serde_json::Value, inboxes: Vec<String>)
{
    let client = reqwest::Client::new();

    for inbox in inboxes
    {
        let url = Url::parse(&inbox).unwrap();
        let host = url.host_str().unwrap();
        let signature = http_signatures::sign(&mut actor, String::from(url.path()), String::from(host));

        println!("Federating activity to inbox: {}", inbox);

        let request = client.post(&inbox)
        .header(HOST, host)
        .header(DATE, chrono::Utc::now().to_rfc2822().to_string())
        .header("Signature", signature)
        .json(&activity)
        .send();
    }
}
