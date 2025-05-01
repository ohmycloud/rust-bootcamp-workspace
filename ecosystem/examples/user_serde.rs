use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    name: String,
    age: u8,
    date_of_birth: DateTime<Utc>,
    skills: Vec<String>,
    state: WorkState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum WorkState {
    Starting,
    Working(String),
    OnLeave(DateTime<Utc>),
    Terminated(DateTime<Utc>),
}

fn main() -> Result<()> {
    let state = WorkState::Starting;
    let user = User {
        name: "John Doe".to_string(),
        age: 30,
        date_of_birth: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
        state,
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
