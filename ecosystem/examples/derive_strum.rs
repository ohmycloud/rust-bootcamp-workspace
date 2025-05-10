use anyhow::Result;
use strum::{
    Display, EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoStaticStr,
    VariantNames,
};

// You need to bring the ToString trait into scope to use it
use std::string::ToString;

#[allow(dead_code)]
#[derive(
    Debug, EnumCount, EnumDiscriminants, EnumIs, EnumIter, EnumString, IntoStaticStr, VariantNames,
)]
enum MyEnum {
    A,
    B(String),
    C,
}

#[derive(Display, Debug)]
enum Color {
    #[strum(serialize = "redred")]
    Red,
    Green {
        range: usize,
    },
    Blue(usize),
    Yellow,
    #[strum(to_string = "purple with {sat} saturation")]
    Purple {
        sat: usize,
    },
}

fn enum_display() {
    // uses the serialize string for Display
    let red = Color::Red;
    assert_eq!(String::from("redred"), format!("{}", red));
    // by default the variants Name
    let yellow = Color::Yellow;
    assert_eq!(String::from("Yellow"), yellow.to_string());
    // or for string formatting
    assert_eq!(
        "blue: Blue green: Green",
        format!(
            "blue: {} green: {}",
            Color::Blue(10),
            Color::Green { range: 42 }
        )
    );
    // you can also use named fields in message
    let purple = Color::Purple { sat: 10 };
    assert_eq!(
        String::from("purple with 10 saturation"),
        purple.to_string()
    );
}

fn main() -> Result<()> {
    println!("{:?}", MyEnum::VARIANTS);
    MyEnum::VARIANTS.iter().for_each(|v| println!("{}", v));

    let my_enum = MyEnum::B("hello".to_string());
    println!("{:?}", my_enum.is_b());

    let s: &'static str = my_enum.into();
    println!("{:?}", s);

    enum_display();
    Ok(())
}
