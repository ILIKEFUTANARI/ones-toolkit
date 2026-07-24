use serde_json::Value;

pub(super) fn display_value(value: &Value, value_type: Option<&str>) -> Option<String> {
    match value {
        Value::Null => None,
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => {
            if value_type == Some("datetime") {
                format_timestamp(&Value::Number(value.clone()))
            } else {
                Some(value.to_string())
            }
        }
        Value::String(value) => {
            let value = value.trim();
            (!value.is_empty()).then(|| html_to_text(value))
        }
        Value::Array(values) => {
            let values: Vec<_> = values
                .iter()
                .filter_map(|value| display_value(value, None))
                .collect();
            (!values.is_empty()).then(|| values.join(", "))
        }
        Value::Object(_) => string_field(value, "name")
            .or_else(|| string_field(value, "label"))
            .or_else(|| string_field(value, "value")),
    }
}

pub(crate) fn format_timestamp(value: &Value) -> Option<String> {
    let raw = value.as_i64()?;
    let seconds = if raw.abs() >= 10_000_000_000_000 {
        raw / 1_000_000
    } else {
        raw / 1_000
    };
    let days = seconds.div_euclid(86_400);
    let time = seconds.rem_euclid(86_400);
    let (year, month, day) = civil_date(days);
    Some(format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        time / 3600,
        (time % 3600) / 60,
        time % 60
    ))
}

fn civil_date(days_since_epoch: i64) -> (i64, i64, i64) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    (
        year + (mp >= 10) as i64,
        mp + if mp < 10 { 3 } else { -9 },
        doy - (153 * mp + 2) / 5 + 1,
    )
}

pub(super) fn html_to_text(input: &str) -> String {
    let mut text = String::with_capacity(input.len());
    let mut tag = String::new();
    let mut in_tag = false;

    for character in input.chars() {
        match character {
            '<' if !in_tag => {
                in_tag = true;
                tag.clear();
            }
            '>' if in_tag => {
                if is_block_tag(&tag) {
                    text.push(' ');
                }
                in_tag = false;
            }
            _ if in_tag => tag.push(character),
            _ => text.push(character),
        }
    }

    text.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn is_block_tag(tag: &str) -> bool {
    let name = tag
        .trim()
        .trim_start_matches('/')
        .split(|character: char| character.is_ascii_whitespace() || character == '/')
        .next()
        .unwrap_or_default();
    [
        "address",
        "article",
        "aside",
        "blockquote",
        "br",
        "div",
        "footer",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hr",
        "li",
        "main",
        "nav",
        "ol",
        "p",
        "pre",
        "section",
        "table",
        "td",
        "th",
        "tr",
        "ul",
    ]
    .iter()
    .any(|candidate| name.eq_ignore_ascii_case(candidate))
}

pub(super) fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(super) fn object_field(value: &Value, key: &str) -> Option<Value> {
    value.get(key).cloned().filter(Value::is_object)
}

pub(super) fn array_field(value: &Value, key: &str) -> Vec<Value> {
    value
        .get(key)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn formats_datetime_and_html() {
        assert_eq!(
            format_timestamp(&json!(0)).as_deref(),
            Some("1970-01-01T00:00:00Z")
        );
        assert_eq!(html_to_text("<p>Hello&nbsp;<b>ONES</b></p>"), "Hello ONES");
        assert_eq!(
            html_to_text("<p>First</p><p>Second<br>line</p>"),
            "First Second line"
        );
    }
}
