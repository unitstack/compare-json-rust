use serde_json::Value;

pub fn path_segments_to_string(segments: &[String]) -> String {
    if segments.is_empty() {
        return String::new();
    }
    let mut result = segments[0].clone();
    for seg in &segments[1..] {
        if is_index_segment(seg) {
            result.push_str(seg);
        } else {
            result.push('.');
            result.push_str(seg);
        }
    }
    result
}

/// Matches `[<digits>]` only (e.g. `[0]`, `[12]`); object keys like `[]`,
/// `[x]` or `[1][2]` are NOT index segments and get a dot separator.
fn is_index_segment(seg: &str) -> bool {
    match seg.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        Some(inner) => !inner.is_empty() && inner.bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

/// Mirrors the TS core `getValueType` (`compare-json-core/src/utils.ts`).
/// serde_json has no `undefined` value, so it is never returned.
pub fn get_value_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn get_value_type_covers_all_variants() {
        assert_eq!(get_value_type(&json!(null)), "null");
        assert_eq!(get_value_type(&json!(true)), "boolean");
        assert_eq!(get_value_type(&json!(1)), "number");
        assert_eq!(get_value_type(&json!(1.5)), "number");
        assert_eq!(get_value_type(&json!("s")), "string");
        assert_eq!(get_value_type(&json!([])), "array");
        assert_eq!(get_value_type(&json!({})), "object");
    }

    #[test]
    fn path_segments_to_string_formats_paths() {
        assert_eq!(path_segments_to_string(&[]), "");
        assert_eq!(path_segments_to_string(&["a".to_string()]), "a");
        assert_eq!(
            path_segments_to_string(&["users".to_string(), "[0]".to_string(), "name".to_string()]),
            "users[0].name"
        );
        assert_eq!(
            path_segments_to_string(&[
                "root".to_string(),
                "[123]".to_string(),
                "key".to_string(),
                "456".to_string(),
                "subkey".to_string()
            ]),
            "root[123].key.456.subkey"
        );
    }
}
