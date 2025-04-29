use std::fs;
use std::mem::size_of;

use anyhow::Context;
use ecosystem::MyError;

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}

fn size_of_error() {
    println!("Size of anyhow::Error is {}", size_of::<anyhow::Error>());
    println!("Size of MyError: {}", size_of::<MyError>());
    println!("Size of std::io::Error: {}", size_of::<std::io::Error>());
    println!(
        "Size of std::num::ParseIntError: {}",
        size_of::<std::num::ParseIntError>()
    );
    println!(
        "Size of serde_json::Error is {}",
        size_of::<serde_json::Error>()
    );
    println!("Size of String is {}", size_of::<String>());
}

fn main() -> Result<(), anyhow::Error> {
    size_of_error();

    let filename = "test.raku";
    let _fd =
        fs::File::open(filename).with_context(|| format!("Failed to open file `{}`", filename))?;
    fail_with_error()?;

    Ok(())
}
