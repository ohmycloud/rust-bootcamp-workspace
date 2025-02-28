#![allow(unused)]
#![allow(clippy::unused_enumerate_index)]

use anyhow::Context;

fn main() -> anyhow::Result<()> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    let file = File::open("data/nginx.log").context("Failed to open log file")?;
    let reader = BufReader::new(file);

    for (_index, line) in reader.lines().enumerate() {
        let log_line = line.context("Failed to read line")?;
        // match parse_nginx_log(&log_line) {
        //     Ok(log) => println!("Line {}: {:#?}", index + 1, log),
        //     Err(e) => eprintln!("Error parsing line {}: {}", index + 1, e),
        // }
        println!("{}", log_line);
    }

    Ok(())
}
