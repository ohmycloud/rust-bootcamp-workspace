use anyhow::Result;
use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    #[serde(rename = "privateAge")]
    age: u8,
    date_of_birth: DateTime<Utc>,
    skills: Vec<String>,
    state: WorkState,
    #[serde(serialize_with = "b64_encode", deserialize_with = "b64_decode")]
    data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

fn main() -> Result<()> {
    let state = WorkState::Starting;
    let user = User {
        name: "John Doe".to_string(),
        age: 30,
        date_of_birth: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
        state,
        data: vec![1, 2, 3, 4, 5],
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
