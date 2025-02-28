mod base64;
pub mod csv;
pub mod genpass;
pub mod text;

pub use self::{
    base64::Base64Format, base64::Base64SubCommand, csv::OutputFormat, text::TextSignFormat,
    text::TextSignOpts,
};
use self::{csv::CsvOpts, genpass::GenPassOpts};
use crate::cli::text::TextSubCommand;
use clap::Parser;
use std::path::{Path, PathBuf};

// 检查输入路径是否存在
pub fn verify_file(input_path: &str) -> Result<String, &'static str> {
    if input_path == "-" || std::path::Path::new(input_path).exists() {
        Ok(input_path.into())
    } else {
        Err("file doesn't exist")
    }
}

pub fn verify_path(path: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(path);
    if p.exists() && p.is_dir() {
        Ok(p.into())
    } else {
        Err("File doesn't exist")
    }
}

#[derive(Debug, Parser)]
pub enum SubCommand {
    #[command(name = "csv", about = "Show CSV, or convert CSV to other formats")]
    Csv(CsvOpts),
    #[command(name = "genpass", about = "Generate a random password")]
    GenPass(GenPassOpts),
    #[command(subcommand)]
    Base64(Base64SubCommand),
    #[command(subcommand)]
    Text(TextSubCommand),
}

#[derive(Debug, Parser)]
#[command(name = "rcli", version, author, about, long_about = None)]
pub struct Opts {
    #[command(subcommand)]
    pub cmd: SubCommand,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_input_file() {
        assert_eq!(verify_file("-"), Ok("-".into()));
        assert_eq!(verify_file("Cargo.toml"), Ok("Cargo.toml".into()));
        assert_eq!(verify_file("non-exist"), Err("file doesn't exist"));
    }
}
