use anyhow::Result;
use derive_builder::Builder;

#[derive(Debug, Builder)]
struct User {
    #[builder(setter(into))]
    name: String,
    #[builder(default = 42)]
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
        .build()?;
    println!("{:?}", user);

    Ok(())
}
