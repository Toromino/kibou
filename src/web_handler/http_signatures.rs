use actor::Actor;

fn get_default_header(key_id: &str, signature: &str) -> &'static str
{
    Box::leak(format!("keyId=\"{}\",headers=\"(request-target) host date\",signature=\"{}\"", &key_id, &signature).into_boxed_str())
}

pub fn sign(actor: &mut Actor, target: String, host: String) -> &'static str
{
    let request: String = format!("(request-target): post {}\nhost: {}\ndate: {}", target, host, chrono::Utc::now().to_rfc2822().to_string());
    let signature: String = actor.sign(request);
    get_default_header(&format!("{}#main-key", actor.actor_uri), &signature)
}
