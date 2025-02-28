use anyhow::Result;
use log_parser::parser::pjson::{parse_value, JsonParser, JsonValue, Rule};
use pest::consumes_to;
use pest::{parses_to, Parser};

#[test]
fn pest_parse_null_should_work() -> Result<()> {
    let input = "null";
    let parsed = JsonParser::parse(Rule::null, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(JsonValue::Null, result);
    Ok(())
}

#[test]
fn pest_parse_bool_should_work() -> Result<()> {
    let input = "true";
    let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(JsonValue::Bool(true), result);

    let input = "false";
    let parsed = JsonParser::parse(Rule::bool, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(JsonValue::Bool(false), result);

    Ok(())
}

#[test]
fn pest_parse_number_should_work() -> Result<()> {
    let input = "123";
    let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(JsonValue::Number(123.0), result);

    let input = "-123";
    let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(JsonValue::Number(-123.0), result);

    let input = "-123.456";
    let parsed = JsonParser::parse(Rule::number, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(JsonValue::Number(-123.456), result);

    Ok(())
}

#[test]
fn pest_parse_string_should_work() -> Result<()> {
    let input = r#""rakulang \"rocks\"""#;
    let parsed = JsonParser::parse(Rule::string, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(
        JsonValue::String(r#"rakulang \"rocks\""#.to_string()),
        result
    );

    Ok(())
}

#[test]
fn pest_parse_array_should_work() -> Result<()> {
    let input = r#"[25, 26, 27]"#;
    let parsed = JsonParser::parse(Rule::array, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(
        JsonValue::Array(vec![
            JsonValue::Number(25.0),
            JsonValue::Number(26.0),
            JsonValue::Number(27.0),
        ]),
        result
    );
    Ok(())
}

#[test]
fn pest_parse_object_should_work() -> Result<()> {
    let input = r#"{"language": "rakulang", "version": "6.e", "vvv": 6}"#;
    let parsed = JsonParser::parse(Rule::object, input)?.next().unwrap();
    let result = parse_value(parsed)?;
    assert_eq!(
        JsonValue::Object(
            vec![
                ("language".to_string(), JsonValue::String("rakulang".into())),
                ("version".to_string(), JsonValue::String("6.e".into())),
                ("vvv".to_string(), JsonValue::Number(6.0)),
            ]
            .into_iter()
            .collect()
        ),
        result
    );
    Ok(())
}

#[test]
fn pest_parse_rule_should_work() {
    parses_to! {
        parser: JsonParser,
        input: r#"{ "hello": "world" }"#,
        rule: Rule::json,
        tokens: [
            object(0, 20, [
                pair(2, 18, [
                    chars(3, 8),
                    value(11, 18, [
                        chars(12, 17)
                    ])
                ])
            ])
        ]
    }
}
