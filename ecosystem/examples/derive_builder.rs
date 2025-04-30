use anyhow::Result;
use chrono::{DateTime, Utc};
use derive_builder::Builder;

#[derive(Debug, Builder)]
struct User {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into, strip_option), default)]
    email: Option<String>,
    #[builder(setter(custom))]
    dob: DateTime<Utc>,
    #[builder(default = "20")]
    age: u32,
    #[builder(default = vec![], setter(each(name = "skill", into)))]
    skills: Vec<String>,
}

impl User {
    pub fn build() -> UserBuilder {
        UserBuilder::default()
    }
}

impl UserBuilder {
    pub fn dob(&mut self, dob: &str) -> &mut Self {
        self.dob = Some(
            DateTime::parse_from_rfc3339(dob)
                .unwrap()
                .with_timezone(&Utc),
        );
        self
    }
}

fn main() -> Result<()> {
    let user = User::build()
        .name("Alice")
        .skill("书法")
        .skill("音乐")
        .email("ohmycloudy@uk")
        .dob("1994-07-05T08:00:00Z")
        .build()?;
    println!("{:?}", user);

    Ok(())
}
