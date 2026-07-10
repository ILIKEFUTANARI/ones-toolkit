use serde_json::Value;
use std::collections::BTreeSet;

pub fn keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect()
}

pub fn assert_no_nulls(value: &Value) {
    match value {
        Value::Null => panic!("response contains null: {value}"),
        Value::Array(values) => values.iter().for_each(assert_no_nulls),
        Value::Object(values) => values.values().for_each(assert_no_nulls),
        _ => {}
    }
}
