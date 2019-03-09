pub mod federator;
pub mod http_signatures;

use reqwest::header::ACCEPT;
use reqwest::header::HeaderValue;

pub fn fetch_remote_object(url: &str) -> Result<String, reqwest::Error>
{
    let client = reqwest::Client::new();
    let request = client.get(url)
    .header(ACCEPT, HeaderValue::from_static("application/activity+json"))
    .send();

    match request
    {
        Ok(mut req) => req.text(),
        Err(req) => Err(req)

    }
}
