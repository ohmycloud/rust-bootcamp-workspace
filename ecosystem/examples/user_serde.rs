use std::{fmt, str::FromStr};

use anyhow::Result;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chacha20poly1305::{
    AeadCore, ChaCha20Poly1305, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::{DisplayFromStr, serde_as};

const KEY: &[u8] = b"01234567890123456789012345678901";

#[derive(Debug)]
struct Sensitive(String);

impl fmt::Display for Sensitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let encrypted = encrypt(self.0.as_bytes()).unwrap();
        write!(f, "{}", encrypted)
    }
}

impl FromStr for Sensitive {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let decrypted = decrypt(s)?;
        let decrypted = String::from_utf8(decrypted)?;
        Ok(Self(decrypted))
    }
}

impl Sensitive {
    fn new(data: impl Into<String>) -> Self {
        Self(data.into())
    }
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    #[serde(rename = "privateAge")]
    age: u8,
    date_of_birth: DateTime<Utc>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
    // #[serde(serialize_with = "serde_encrypt", deserialize_with = "serde_decrypt")]
    #[serde_as(as = "DisplayFromStr")]
    sensitive: Sensitive,
    #[serde_as(as = "Vec<DisplayFromStr>")]
    url: Vec<http::Uri>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "details")]
enum WorkState {
    Starting,
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminated(DateTime<Utc>),
}

fn b64_encode<S>(data: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encoded = BASE64_URL_SAFE_NO_PAD.encode(data);
    serializer.serialize_str(&encoded)
}

fn b64_decode<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encoded = String::deserialize(deserializer)?;
    let decoded = BASE64_URL_SAFE_NO_PAD
        .decode(encoded.as_bytes())
        .map_err(serde::de::Error::custom)?;
    Ok(decoded)
}

// encrypt with chacha20poly1305 and then encode with base64
fn encrypt(data: &[u8]) -> Result<String> {
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data).unwrap();
    let nonce_cypertext: Vec<_> = nonce.iter().copied().chain(ciphertext).collect();
    let encoded = BASE64_URL_SAFE_NO_PAD.encode(nonce_cypertext);

    Ok(encoded)
}

fn decrypt(encoded: &str) -> Result<Vec<u8>> {
    let decoded = BASE64_URL_SAFE_NO_PAD.decode(encoded.as_bytes())?;
    let cipher = ChaCha20Poly1305::new(KEY.into());
    let nonce_bytes = &decoded[..12];
    let nonce = Nonce::from_slice(nonce_bytes);
    let decrypted = cipher.decrypt(&nonce, &decoded[12..]).unwrap();
    Ok(decrypted)
}

fn serde_encrypt<S>(data: &str, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let encrypted = encrypt(data.as_bytes()).map_err(serde::ser::Error::custom)?;
    serializer.serialize_str(&encrypted)
}

fn serde_decrypt<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let encrypted = String::deserialize(deserializer)?;
    let decrypted = decrypt(&encrypted).map_err(serde::de::Error::custom)?;
    let decrypted = String::from_utf8(decrypted).map_err(serde::de::Error::custom)?;
    Ok(decrypted)
}

fn main() -> Result<()> {
    let state = WorkState::Starting;
    let user = User {
        name: "John Doe".to_string(),
        age: 30,
        date_of_birth: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
        state,
        data: vec![1, 2, 3, 4, 5],
        sensitive: Sensitive::new("Sensitive Data"),
        url: vec!["https://example.com".parse()?],
    };

    let json = serde_json::to_string(&user)?;
    println!("{}", json);
    let user: User = serde_json::from_str(&json)?;
    println!("{:?}", user);

    let state = WorkState::OnLeave(Utc::now());
    let json = serde_json::to_string(&state)?;
    println!("{:?}", json);
    let state: WorkState = serde_json::from_str(&json)?;
    println!("{:?}", state);

    Ok(())
}
