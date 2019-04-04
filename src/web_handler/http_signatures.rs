use actor::Actor;
use base64;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use std::collections::HashMap;

#[derive(Debug)]
pub struct HTTPSignature {
    pub content_length: String,
    pub date: String,
    pub digest: String,
    pub endpoint: String,
    pub host: String,
    pub signature: String,
}

fn get_default_header(key_id: &str, signature: &str) -> &'static str {
    Box::leak(
        format!(
            "keyId=\"{}\",headers=\"(request-target) host date\",signature=\"{}\"",
            &key_id, &signature
        )
        .into_boxed_str(),
    )
}

pub fn sign(actor: &mut Actor, target: String, host: String) -> &'static str {
    let request: String = format!(
        "(request-target): post {}\nhost: {}\ndate: {}",
        target,
        host,
        chrono::Utc::now().to_rfc2822().to_string()
    );
    let signature: String = actor.sign(request);
    get_default_header(&format!("{}#main-key", actor.actor_uri), &signature)
}

pub fn validate(actor: &mut Actor, sig_headers: HTTPSignature) -> bool {
    let mut parsed_signature = String::new();
    let stripped_characters = "\"";
    for character in sig_headers.signature.chars() {
        if !stripped_characters.contains(character) {
            parsed_signature.push(character);
        }
    }

    let header_tags: HashMap<String, String> = parsed_signature
        .split(',')
        .map(|kv| kv.split('='))
        .map(|mut kv| (kv.next().unwrap().into(), kv.next().unwrap().into()))
        .collect();

    let headers = &header_tags["headers"];

    let inner_signature =
        &base64::decode(&header_tags["signature"].clone().into_bytes()).unwrap();
    let mut new_signature: Vec<String> = vec![];

    if headers.contains("(request-target)") {
        new_signature.push(format!("(request-target): {}", sig_headers.endpoint));
    }

    if headers.contains("content-length") {
        new_signature.push(format!("content-length: {}", sig_headers.content_length));
    }

    if headers.contains("date") {
        new_signature.push(format!("date: {}", sig_headers.date));
    }

    if headers.contains("digest") {
        new_signature.push(format!("digest: {}", sig_headers.digest));
    }

    if headers.contains("host") {
        new_signature.push(format!("host: {}", sig_headers.host));
    }

    let pem_decoded = pem::parse(actor.get_public_key()).unwrap();
    let pkey = PKey::from_rsa(openssl::rsa::Rsa::public_key_from_der(&pem_decoded.contents).unwrap()).unwrap();
    let mut verifier = Verifier::new(MessageDigest::sha256(), &pkey).unwrap();
    verifier
        .update(&new_signature.join("\n").into_bytes())
        .unwrap();

    verifier.verify(inner_signature).unwrap()
}
