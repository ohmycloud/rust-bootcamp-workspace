use anyhow::{anyhow, Result};
use log_parser::parser::pjson::{parse_value, JsonParser, Rule};
use pest::iterators::Pair;
use pest::Parser;

fn main() -> Result<()> {
    let json_str = r#"{
        "name": "John Doe",
        "age": 30,
        "is_student": false,
        "marks": [90.0, -80.0, 85.1],
        "address": {
            "city": "New York",
            "zip": 10001
        }
    }"#;

    let parsed: Pair<Rule> = JsonParser::parse(Rule::json, json_str)?
        .next()
        .ok_or_else(|| anyhow!("json has no value"))?;
    let value = parse_value(parsed);
    println!("{:#?}", value);
    Ok(())
}
