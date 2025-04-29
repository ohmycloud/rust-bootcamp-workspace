use std::fs;

use anyhow::Context;
use ecosystem::MyError;

fn fail_with_error() -> Result<(), MyError> {
    Err(MyError::Custom("This is a custom error".to_string()))
}

fn main() -> Result<(), anyhow::Error> {
    let filename = "test.raku";
    let _fd =
        fs::File::open(filename).with_context(|| format!("Failed to open file `{}`", filename))?;
    fail_with_error()?;

    Ok(())
}
