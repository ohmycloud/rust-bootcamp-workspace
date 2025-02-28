use crate::cli::Base64Format;
use crate::utils::input_reader;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::prelude::*;

pub fn process_encode(input: String, format: Base64Format) -> anyhow::Result<()> {
    let mut reader = input_reader(input)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;

    let encoded = match format {
        Base64Format::Standard => STANDARD.encode(&buf),
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.encode(&buf),
    };
    println!("{}", encoded);
    Ok(())
}

pub fn process_decode(input: String, format: Base64Format) -> anyhow::Result<()> {
    let mut reader = input_reader(input)?;
    let mut buf = String::new();
    reader.read_to_string(&mut buf)?;
    // avoid accident newline
    let buf = buf.trim();

    let decoded = match format {
        Base64Format::Standard => STANDARD.decode(buf)?,
        Base64Format::UrlSafe => URL_SAFE_NO_PAD.decode(buf)?,
    };
    let decoded = String::from_utf8(decoded)?;
    println!("{}", decoded);
    Ok(())
}
