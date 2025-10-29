use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize)]
struct Jwk {
    kty: String,
    #[serde(rename = "use")]
    use_: Option<String>,
    kid: String,
    n: String,
    e: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct JwksResponse {
    keys: Vec<Jwk>,
}
