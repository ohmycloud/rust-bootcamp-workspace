use anyhow::Result;
use derive_builder::Builder;

#[derive(Debug, Builder)]
struct User {
    #[builder(setter(into))]
    name: String,
    #[builder(setter(into, strip_option), default)]
    email: Option<String>,
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

fn main() -> Result<()> {
    let user = User::build()
        .name("Alice")
        .skill("书法")
        .skill("音乐")
        .email("ohmycloudy@uk")
        .build()?;
    println!("{:?}", user);

    Ok(())
}
