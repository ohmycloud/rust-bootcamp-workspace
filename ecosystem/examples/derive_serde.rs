use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, ser::SerializeStruct};

#[derive(Debug, PartialEq, Deserialize)]
struct User {
    name: String,
    age: u8,
    dob: DateTime<Utc>,
    skills: Vec<String>,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("User", 4)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("age", &self.age)?;
        state.serialize_field("dob", &self.dob)?;
        state.serialize_field("skills", &self.skills)?;
        state.end()
    }
}

fn main() {
    let user = User {
        name: "John Doe".to_string(),
        age: 30,
        dob: Utc::now(),
        skills: vec!["Rust".to_string(), "Python".to_string()],
    };

    let json = serde_json::to_string(&user);
    println!("{:?}", json);
}
