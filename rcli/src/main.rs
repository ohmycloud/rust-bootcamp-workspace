use clap::Parser;
use rcli::cli::text::TextSubCommand;
use rcli::cli::{Base64SubCommand, TextSignFormat};
use rcli::process::{
    process_decode, process_encode, process_text_generate, process_text_sign, process_text_verify,
};
use rcli::{cli::Opts, cli::SubCommand, process::process_csv, process::process_genpass};
use std::fs;

fn main() -> anyhow::Result<()> {
    let cli = Opts::parse();
    match cli.cmd {
        SubCommand::Csv(opts) => {
            let output = if let Some(output) = opts.output {
                output.clone()
            } else {
                format!("output.{}", opts.format)
            };
            process_csv(&opts.input, output, opts.format)?;
        }
        SubCommand::GenPass(opts) => {
            let password = process_genpass(
                opts.length,
                opts.lowercase,
                opts.uppercase,
                opts.number,
                opts.symbol,
            );
            if let Ok(password) = password {
                println!("{}", password);
            }
        }
        SubCommand::Base64(subcmd) => match subcmd {
            Base64SubCommand::Encode(opts) => {
                process_encode(opts.input, opts.format)?;
            }
            Base64SubCommand::Decode(opts) => {
                process_decode(opts.input, opts.format)?;
            }
        },
        SubCommand::Text(subcmd) => match subcmd {
            TextSubCommand::Sign(opts) => {
                let sig = process_text_sign(opts.input, opts.key, opts.format)?;
                println!("sig: {}", sig);
            }
            TextSubCommand::Verify(opts) => {
                let verified = process_text_verify(opts.input, opts.key, opts.format, opts.sig)?;
                println!("verified: {}", verified);
            }
            TextSubCommand::Generate(opts) => {
                let key = process_text_generate(opts.format)?;
                match opts.format {
                    TextSignFormat::Blake3 => {
                        let filename = opts.output.join("blake3.txt");
                        fs::write(filename, &key[0])?;
                    }
                    TextSignFormat::Ed25519 => {
                        let filename = &opts.output;
                        fs::write(filename.join("ed25519.sk"), &key[0])?;
                        fs::write(filename.join("ed25519.pk"), &key[1])?;
                    }
                }
            }
        },
    }
    Ok(())
}
