use actor::Actor;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::sign::Verifier;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Signature {
    pub algorithm: Option<String>,
    pub content_length: Option<String>,
    pub content_type: Option<String>,
    pub date: String,
    pub digest: Option<String>,
    pub host: String,
    pub key_id: Option<String>,
    pub headers: Vec<String>,
    pub request_target: Option<String>,
    pub signature: String,
    pub signature_in_bytes: Option<Vec<u8>>,
}

impl Signature {
    pub fn build_header(self) -> String {
        format!("keyId=\"{key_id}\",algorithm=\"{algorithm}\",headers=\"{headers}\",signature=\"{signature}\"",
        key_id = self.key_id.unwrap(),
        algorithm = self.algorithm.unwrap(),
        headers = self.headers.join(" "),
        signature = self.signature
        )
    }

    pub fn verify(self, actor: &mut Actor) -> bool {
        let mut signature: Vec<String> = Vec::new();

        for header in self.headers {
            match header.as_str() {
                "(request-target)" => signature.push(format!(
                    "(request_target): post {}",
                    self.request_target.as_ref().unwrap()
                )),
                "content-type" => signature.push(format!(
                    "content-type: {}",
                    self.content_type.as_ref().unwrap()
                )),
                "content-length" => signature.push(format!(
                    "content-length: {}",
                    self.content_length.as_ref().unwrap()
                )),
                "date" => signature.push(format!("date: {}", &self.date)),
                "digest" => signature.push(format!("digest: {}", self.digest.as_ref().unwrap())),
                "host" => signature.push(format!("host: {}", &self.host)),
                _ => (),
            }
        }

        let decoded_pem = pem::parse(actor.get_public_key()).unwrap();
        let public_key =
            PKey::from_rsa(Rsa::public_key_from_der(&decoded_pem.contents).unwrap()).unwrap();

        let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key).unwrap();
        verifier.update(&signature.join("\n").into_bytes()).unwrap();

        return verifier.verify(&self.signature_in_bytes.unwrap()).unwrap();
    }

    pub fn new(key_id: &str, request_target: &str, host: &str) -> Signature {
        return Signature {
            algorithm: Some(String::from("rsa-sha256")),
            content_type: None,
            content_length: None,
            date: chrono::Utc::now().to_rfc2822().to_string(),
            digest: None,
            host: host.into(),
            key_id: Some(key_id.into()),
            headers: vec![
                "(request-target)".to_string(),
                "date".to_string(),
                "host".to_string(),
            ],
            request_target: Some(request_target.into()),
            signature: String::from(""),
            signature_in_bytes: None,
        };
    }

    pub fn sign(&mut self, actor: &mut Actor) {
        let string_to_be_signed = format!(
            "(request-target): post {request_target}\ndate: {date}\nhost: {host}",
            request_target = self.request_target.as_ref().unwrap(),
            date = self.date,
            host = self.host
        );

        self.signature = actor.sign(string_to_be_signed);
    }
}
