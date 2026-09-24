use serde_json::Value;
use crate::types::{CompareOptions, Difference, DiffType, PathBelongsTo, ArrayCompareMethod};
use crate::utils::path_segments_to_string;

pub fn compare_json(base: &Value, contrast: &Value, options: &CompareOptions) -> Vec<Difference> {
    compare_value(base, contrast, &[], options)
}

fn compare_value(base: &Value, contrast: &Value, path: &[String], options: &CompareOptions) -> Vec<Difference> {
    match (base, contrast) {
        (Value::Object(s), Value::Object(t)) => compare_object(s, t, path, options),
        (Value::Array(s), Value::Array(t)) => {
            match options.array_compare_method {
                Some(ArrayCompareMethod::Lcs) => compare_array_lcs(s, t, path, options),
                Some(ArrayCompareMethod::Unordered) => compare_array_unordered(s, t, path, options),
                _ => compare_array(s, t, path, options),
            }
        }
        _ if !same_type(base, contrast) => {
            if options.numeric_string_equals_number && is_numeric_match(base, contrast) {
                vec![]
            } else {
                vec![Difference {
                    path_segments: path.to_vec(),
                    path_string: path_segments_to_string(path),
                    path_belongs_to: PathBelongsTo::Both,
                    diff_type: DiffType::TypeChanged,
                }]
            }
        }
        _ if !values_equal(base, contrast, options) => {
            vec![Difference {
                path_segments: path.to_vec(),
                path_string: path_segments_to_string(path),
                path_belongs_to: PathBelongsTo::Both,
                diff_type: DiffType::ValueChanged,
            }]
        }
        _ => vec![],
    }
}

fn compare_object(base: &serde_json::Map<String, Value>, contrast: &serde_json::Map<String, Value>, path: &[String], options: &CompareOptions) -> Vec<Difference> {
    let mut diffs = Vec::new();
    let mut matched = std::collections::HashSet::new();

    let contrast_key_map: Option<std::collections::HashMap<String, Vec<&String>>> = if options.key_case_insensitive {
        let mut map: std::collections::HashMap<String, Vec<&String>> = std::collections::HashMap::new();
        for key in contrast.keys() {
            map.entry(key.to_lowercase()).or_default().push(key);
        }
        Some(map)
    } else {
        None
    };

    for (key, val) in base {
        let target_key = if options.key_case_insensitive {
            let lower = key.to_lowercase();
            contrast_key_map.as_ref().and_then(|m| {
                m.get(&lower).and_then(|candidates| {
                    candidates.iter().find(|k| !matched.contains(**k)).map(|k| (*k).clone())
                })
            })
        } else {
            if contrast.contains_key(key) { Some(key.clone()) } else { None }
        };

        let mut new_path = path.to_vec();
        new_path.push(key.clone());

        if let Some(tk) = target_key {
            matched.insert(tk.clone());
            diffs.extend(compare_value(val, &contrast[&tk], &new_path, options));
        } else {
            diffs.push(Difference {
                path_segments: new_path.clone(),
                path_string: path_segments_to_string(&new_path),
                path_belongs_to: PathBelongsTo::Base,
                diff_type: DiffType::Deleted,
            });
        }
    }

    for key in contrast.keys() {
        if matched.contains(key) {
            continue;
        }
        let has_match = if options.key_case_insensitive {
            // Same Unicode-aware comparison as the first pass (to_lowercase),
            // so both passes agree on which keys match.
            base.keys().any(|k| k.to_lowercase() == key.to_lowercase())
        } else {
            base.contains_key(key)
        };
        if !has_match {
            let mut new_path = path.to_vec();
            new_path.push(key.clone());
            diffs.push(Difference {
                path_segments: new_path.clone(),
                path_string: path_segments_to_string(&new_path),
                path_belongs_to: PathBelongsTo::Contrast,
                diff_type: DiffType::Added,
            });
        }
    }

    diffs
}

fn compare_array(base: &[Value], contrast: &[Value], path: &[String], options: &CompareOptions) -> Vec<Difference> {
    let mut diffs = Vec::new();
    let min_len = base.len().min(contrast.len());

    for i in 0..min_len {
        let mut new_path = path.to_vec();
        new_path.push(format!("[{}]", i));
        diffs.extend(compare_value(&base[i], &contrast[i], &new_path, options));
    }

    for i in min_len..base.len() {
        let mut new_path = path.to_vec();
        new_path.push(format!("[{}]", i));
        diffs.push(Difference {
            path_segments: new_path.clone(),
            path_string: path_segments_to_string(&new_path),
            path_belongs_to: PathBelongsTo::Base,
            diff_type: DiffType::Deleted,
        });
    }

    for i in min_len..contrast.len() {
        let mut new_path = path.to_vec();
        new_path.push(format!("[{}]", i));
        diffs.push(Difference {
            path_segments: new_path.clone(),
            path_string: path_segments_to_string(&new_path),
            path_belongs_to: PathBelongsTo::Contrast,
            diff_type: DiffType::Added,
        });
    }

    diffs
}

fn compare_array_lcs(base: &[Value], contrast: &[Value], path: &[String], options: &CompareOptions) -> Vec<Difference> {
    let mut diffs = Vec::new();
    let (base_matched, contrast_matched) = compute_lcs(base, contrast, options);

    for i in 0..base.len() {
        if !base_matched.contains(&i) {
            let mut new_path = path.to_vec();
            new_path.push(format!("[{}]", i));
            diffs.push(Difference {
                path_segments: new_path.clone(),
                path_string: path_segments_to_string(&new_path),
                path_belongs_to: PathBelongsTo::Base,
                diff_type: DiffType::Deleted,
            });
        }
    }

    for i in 0..contrast.len() {
        if !contrast_matched.contains(&i) {
            let mut new_path = path.to_vec();
            new_path.push(format!("[{}]", i));
            diffs.push(Difference {
                path_segments: new_path.clone(),
                path_string: path_segments_to_string(&new_path),
                path_belongs_to: PathBelongsTo::Contrast,
                diff_type: DiffType::Added,
            });
        }
    }

    diffs
}

fn compare_array_unordered(base: &[Value], contrast: &[Value], path: &[String], options: &CompareOptions) -> Vec<Difference> {
    let mut diffs = Vec::new();
    let mut matched = vec![false; contrast.len()];

    for (i, b) in base.iter().enumerate() {
        let mut found = false;
        for (j, c) in contrast.iter().enumerate() {
            if !matched[j] && deep_equal(b, c, options) {
                matched[j] = true;
                found = true;
                break;
            }
        }
        if !found {
            let mut new_path = path.to_vec();
            new_path.push(format!("[{}]", i));
            diffs.push(Difference {
                path_segments: new_path.clone(),
                path_string: path_segments_to_string(&new_path),
                path_belongs_to: PathBelongsTo::Base,
                diff_type: DiffType::Deleted,
            });
        }
    }

    for (j, is_matched) in matched.iter().enumerate() {
        if !is_matched {
            let mut new_path = path.to_vec();
            new_path.push(format!("[{}]", j));
            diffs.push(Difference {
                path_segments: new_path.clone(),
                path_string: path_segments_to_string(&new_path),
                path_belongs_to: PathBelongsTo::Contrast,
                diff_type: DiffType::Added,
            });
        }
    }

    diffs
}

fn compute_lcs(base: &[Value], contrast: &[Value], options: &CompareOptions) -> (Vec<usize>, Vec<usize>) {
    let m = base.len();
    let n = contrast.len();
    let mut dp = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if deep_equal(&base[i - 1], &contrast[j - 1], options) {
                dp[i][j] = dp[i - 1][j - 1] + 1;
            } else {
                dp[i][j] = dp[i - 1][j].max(dp[i][j - 1]);
            }
        }
    }

    let mut base_matched = Vec::new();
    let mut contrast_matched = Vec::new();
    let (mut i, mut j) = (m, n);

    while i > 0 && j > 0 {
        if deep_equal(&base[i - 1], &contrast[j - 1], options) {
            base_matched.push(i - 1);
            contrast_matched.push(j - 1);
            i -= 1;
            j -= 1;
        } else if dp[i - 1][j] > dp[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }

    base_matched.reverse();
    contrast_matched.reverse();
    (base_matched, contrast_matched)
}

fn deep_equal(a: &Value, b: &Value, options: &CompareOptions) -> bool {
    compare_value(a, b, &[], options).is_empty()
}

fn same_type(a: &Value, b: &Value) -> bool {
    matches!((a, b),
        (Value::Null, Value::Null) |
        (Value::Bool(_), Value::Bool(_)) |
        (Value::Number(_), Value::Number(_)) |
        (Value::String(_), Value::String(_)) |
        (Value::Array(_), Value::Array(_)) |
        (Value::Object(_), Value::Object(_))
    )
}

fn is_numeric_match(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::String(s), Value::Number(n)) => numeric_string_equals(s, n),
        (Value::Number(n), Value::String(s)) => numeric_string_equals(s, n),
        _ => false,
    }
}

fn numeric_string_equals(s: &str, n: &serde_json::Number) -> bool {
    match (s.parse::<f64>().ok(), n.as_f64()) {
        (Some(v), Some(nv)) => v == nv,
        _ => false,
    }
}

fn values_equal(a: &Value, b: &Value, options: &CompareOptions) -> bool {
    if options.value_case_insensitive {
        if let (Value::String(s1), Value::String(s2)) = (a, b) {
            return s1.to_lowercase() == s2.to_lowercase();
        }
    }
    if let (Value::Number(n1), Value::Number(n2)) = (a, b) {
        return numbers_equal(n1, n2);
    }
    a == b
}

/// Compare two JSON numbers by numeric value, not by internal representation:
/// `1` (integer) and `1.0` (float) are the same JSON number.
fn numbers_equal(a: &serde_json::Number, b: &serde_json::Number) -> bool {
    if a.is_f64() || b.is_f64() {
        match (a.as_f64(), b.as_f64()) {
            (Some(x), Some(y)) => x == y,
            _ => false,
        }
    } else {
        // Both are integers: exact comparison.
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_same_values() {
        let opts = CompareOptions::default();
        assert!(compare_json(&json!("test"), &json!("test"), &opts).is_empty());
        assert!(compare_json(&json!(123), &json!(123), &opts).is_empty());
        assert!(compare_json(&json!(true), &json!(true), &opts).is_empty());
        assert!(compare_json(&json!(null), &json!(null), &opts).is_empty());
    }

    #[test]
    fn test_value_changes() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!("old"), &json!("new"), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Both);
        assert_eq!(diffs[0].diff_type, DiffType::ValueChanged);
    }

    #[test]
    fn test_type_changes() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!("string"), &json!(123), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Both);
        assert_eq!(diffs[0].diff_type, DiffType::TypeChanged);
    }

    #[test]
    fn test_object_deleted_keys() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!({"a": 1, "b": 2}), &json!({"a": 1}), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "b");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[0].diff_type, DiffType::Deleted);
    }

    #[test]
    fn test_object_added_keys() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!({"a": 1}), &json!({"a": 1, "b": 2}), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "b");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Contrast);
        assert_eq!(diffs[0].diff_type, DiffType::Added);
    }

    #[test]
    fn test_nested_object_value_changes() {
        let opts = CompareOptions::default();
        let diffs = compare_json(
            &json!({"nested": {"value": "old"}}),
            &json!({"nested": {"value": "new"}}),
            &opts,
        );
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "nested.value");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Both);
        assert_eq!(diffs[0].diff_type, DiffType::ValueChanged);
    }

    #[test]
    fn test_array_deleted_elements() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!([1, 2, 3]), &json!([1, 2]), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "[2]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[0].diff_type, DiffType::Deleted);
    }

    #[test]
    fn test_array_added_elements() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!([1, 2]), &json!([1, 2, 3]), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "[2]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Contrast);
        assert_eq!(diffs[0].diff_type, DiffType::Added);
    }

    #[test]
    fn test_array_value_changes() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!([1, "old", true]), &json!([1, "new", true]), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "[1]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Both);
        assert_eq!(diffs[0].diff_type, DiffType::ValueChanged);
    }

    #[test]
    fn test_nested_arrays() {
        let opts = CompareOptions::default();
        let diffs = compare_json(&json!([[1, 2], [3, 4]]), &json!([[1, 2], [3, "5"]]), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "[1][1]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Both);
        assert_eq!(diffs[0].diff_type, DiffType::TypeChanged);
    }

    #[test]
    fn test_lcs_shifted_arrays() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Lcs), ..Default::default() };
        let diffs = compare_json(&json!(["a", "b", "c"]), &json!(["b", "c", "d"]), &opts);
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path_string, "[0]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[0].diff_type, DiffType::Deleted);
        assert_eq!(diffs[1].path_string, "[2]");
        assert_eq!(diffs[1].path_belongs_to, PathBelongsTo::Contrast);
        assert_eq!(diffs[1].diff_type, DiffType::Added);
    }

    #[test]
    fn test_lcs_identical_arrays() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Lcs), ..Default::default() };
        assert!(compare_json(&json!([1, 2, 3]), &json!([1, 2, 3]), &opts).is_empty());
    }

    #[test]
    fn test_lcs_empty_arrays() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Lcs), ..Default::default() };
        assert!(compare_json(&json!([]), &json!([]), &opts).is_empty());
    }

    #[test]
    fn test_lcs_no_common_elements() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Lcs), ..Default::default() };
        let diffs = compare_json(&json!([1, 2, 3]), &json!([4, 5, 6]), &opts);
        assert_eq!(diffs.len(), 6);
    }

    #[test]
    fn test_lcs_objects_in_arrays() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Lcs), ..Default::default() };
        let diffs = compare_json(
            &json!([{"id": 1}, {"id": 2}, {"id": 3}]),
            &json!([{"id": 2}, {"id": 3}, {"id": 4}]),
            &opts,
        );
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path_string, "[0]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[1].path_string, "[2]");
        assert_eq!(diffs[1].path_belongs_to, PathBelongsTo::Contrast);
    }

    #[test]
    fn test_lcs_extra_middle_element() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Lcs), ..Default::default() };
        let diffs = compare_json(&json!(["a", "x", "b"]), &json!(["a", "b"]), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "[1]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[0].diff_type, DiffType::Deleted);
    }

    #[test]
    fn test_unordered_reordered_equal() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Unordered), ..Default::default() };
        assert!(compare_json(&json!(["a", "b", "c"]), &json!(["c", "b", "a"]), &opts).is_empty());
    }

    #[test]
    fn test_unordered_added_deleted() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Unordered), ..Default::default() };
        let diffs = compare_json(&json!(["a", "b", "c"]), &json!(["a", "c", "d"]), &opts);
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[0].diff_type, DiffType::Deleted);
        assert_eq!(diffs[1].path_belongs_to, PathBelongsTo::Contrast);
        assert_eq!(diffs[1].diff_type, DiffType::Added);
    }

    #[test]
    fn test_unordered_duplicates() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Unordered), ..Default::default() };
        let diffs = compare_json(&json!(["a", "a", "b"]), &json!(["a", "b", "b"]), &opts);
        assert_eq!(diffs.len(), 2);
    }

    #[test]
    fn test_unordered_objects() {
        let opts = CompareOptions { array_compare_method: Some(ArrayCompareMethod::Unordered), ..Default::default() };
        assert!(compare_json(
            &json!([{"id": 1}, {"id": 2}, {"id": 3}]),
            &json!([{"id": 3}, {"id": 1}, {"id": 2}]),
            &opts,
        ).is_empty());
    }

    #[test]
    fn test_value_case_insensitive() {
        let opts = CompareOptions { value_case_insensitive: true, ..Default::default() };
        assert!(compare_json(&json!("Hello"), &json!("hello"), &opts).is_empty());
        let diffs = compare_json(&json!("hello"), &json!("world"), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::ValueChanged);
    }

    #[test]
    fn test_value_case_insensitive_in_objects() {
        let opts = CompareOptions { value_case_insensitive: true, ..Default::default() };
        assert!(compare_json(&json!({"name": "Alice"}), &json!({"name": "alice"}), &opts).is_empty());
    }

    #[test]
    fn test_value_case_insensitive_in_arrays() {
        let opts = CompareOptions { value_case_insensitive: true, ..Default::default() };
        assert!(compare_json(&json!(["Hello", "World"]), &json!(["hello", "world"]), &opts).is_empty());
    }

    #[test]
    fn test_key_case_insensitive() {
        let opts = CompareOptions { key_case_insensitive: true, ..Default::default() };
        assert!(compare_json(&json!({"Name": "Alice"}), &json!({"name": "Alice"}), &opts).is_empty());
    }

    #[test]
    fn test_key_case_insensitive_nested() {
        let opts = CompareOptions { key_case_insensitive: true, ..Default::default() };
        assert!(compare_json(
            &json!({"User": {"Name": "Alice"}}),
            &json!({"user": {"name": "Alice"}}),
            &opts,
        ).is_empty());
    }

    #[test]
    fn test_key_case_insensitive_deleted() {
        let opts = CompareOptions { key_case_insensitive: true, ..Default::default() };
        let diffs = compare_json(&json!({"a": 1, "B": 2}), &json!({"A": 1}), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "B");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[0].diff_type, DiffType::Deleted);
    }

    #[test]
    fn test_key_case_insensitive_added() {
        let opts = CompareOptions { key_case_insensitive: true, ..Default::default() };
        let diffs = compare_json(&json!({"a": 1}), &json!({"A": 1, "b": 2}), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].path_string, "b");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Contrast);
        assert_eq!(diffs[0].diff_type, DiffType::Added);
    }

    #[test]
    fn test_numeric_string_equals_number() {
        let opts = CompareOptions { numeric_string_equals_number: true, ..Default::default() };
        assert!(compare_json(&json!("123"), &json!(123), &opts).is_empty());
        assert!(compare_json(&json!(456), &json!("456"), &opts).is_empty());
    }

    #[test]
    fn test_numeric_string_non_numeric() {
        let opts = CompareOptions { numeric_string_equals_number: true, ..Default::default() };
        let diffs = compare_json(&json!("abc"), &json!(123), &opts);
        assert_eq!(diffs.len(), 1);
        assert_eq!(diffs[0].diff_type, DiffType::TypeChanged);
    }

    #[test]
    fn test_numeric_string_float() {
        let opts = CompareOptions { numeric_string_equals_number: true, ..Default::default() };
        assert!(compare_json(&json!("3.14"), &json!(3.14), &opts).is_empty());
    }

    #[test]
    fn test_numeric_string_negative() {
        let opts = CompareOptions { numeric_string_equals_number: true, ..Default::default() };
        assert!(compare_json(&json!("-42"), &json!(-42), &opts).is_empty());
    }

    #[test]
    fn test_numeric_string_in_objects() {
        let opts = CompareOptions { numeric_string_equals_number: true, ..Default::default() };
        assert!(compare_json(&json!({"count": "100"}), &json!({"count": 100}), &opts).is_empty());
    }

    #[test]
    fn test_numeric_string_in_arrays() {
        let opts = CompareOptions { numeric_string_equals_number: true, ..Default::default() };
        assert!(compare_json(&json!(["1", "2", "3"]), &json!([1, 2, 3]), &opts).is_empty());
    }

    #[test]
    fn test_numeric_string_unordered() {
        let opts = CompareOptions {
            numeric_string_equals_number: true,
            array_compare_method: Some(ArrayCompareMethod::Unordered),
            ..Default::default()
        };
        assert!(compare_json(&json!(["1", "2", "3"]), &json!([3, 2, 1]), &opts).is_empty());
    }

    #[test]
    fn test_numeric_string_lcs() {
        let opts = CompareOptions {
            numeric_string_equals_number: true,
            array_compare_method: Some(ArrayCompareMethod::Lcs),
            ..Default::default()
        };
        let diffs = compare_json(&json!(["1", "2", "3"]), &json!([2, 3, 4]), &opts);
        assert_eq!(diffs.len(), 2);
        assert_eq!(diffs[0].path_string, "[0]");
        assert_eq!(diffs[0].path_belongs_to, PathBelongsTo::Base);
        assert_eq!(diffs[1].path_string, "[2]");
        assert_eq!(diffs[1].path_belongs_to, PathBelongsTo::Contrast);
    }
}
