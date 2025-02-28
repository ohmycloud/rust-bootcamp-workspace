use anyhow::{anyhow, Result};
use pest::iterators::Pair;
use pest_derive::Parser;
use std::collections::HashMap;

#[derive(Parser)]
#[grammar = "../grammar/json.pest"]
pub struct JsonParser;

#[derive(Debug, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

pub fn parse_array(pair: Pair<Rule>) -> Result<Vec<JsonValue>> {
    pair.into_inner().map(parse_value).collect()
}

pub fn parse_object(pair: Pair<Rule>) -> Result<HashMap<String, JsonValue>> {
    let inner = pair.into_inner();
    let values = inner.map(|pair| {
        let mut inner = pair.into_inner();
        let key = inner
            .next()
            .map(|p| p.as_str().to_string())
            .ok_or_else(|| anyhow!("expected key in object, found none"))?;
        let value = parse_value(
            inner
                .next()
                .ok_or_else(|| anyhow!("expected value in object, found none"))?,
        )?;
        Ok((key, value))
    });
    values.collect::<Result<HashMap<_, _>>>()
}

pub fn parse_value(pair: Pair<Rule>) -> Result<JsonValue> {
    let ret = match pair.as_rule() {
        Rule::null => JsonValue::Null,
        Rule::bool => JsonValue::Bool(pair.as_str().parse()?),
        Rule::number => JsonValue::Number(pair.as_str().parse()?),
        Rule::chars => JsonValue::String(pair.as_str().to_string()),
        Rule::array => JsonValue::Array(parse_array(pair)?),
        Rule::object => JsonValue::Object(parse_object(pair)?),
        Rule::value => {
            let inner = pair
                .into_inner()
                .next()
                .ok_or_else(|| anyhow!("expected value"))?;
            parse_value(inner)?
        }
        v => panic!("unhandled rule: {:?}", v),
    };
    Ok(ret)
}
