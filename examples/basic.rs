use compare_json::{compare_json, CompareOptions, ArrayCompareMethod};
use serde_json::json;

fn main() {
    let source = json!({
        "name": "Alice",
        "age": 30,
        "hobbies": ["reading", "coding"]
    });

    let target = json!({
        "name": "Alice",
        "age": 31,
        "hobbies": ["coding", "gaming"]
    });

    let options = CompareOptions {
        array_compare_method: Some(ArrayCompareMethod::ByIndex),
        ..Default::default()
    };

    let diffs = compare_json(&source, &target, &options);

    println!("Found {} differences:", diffs.len());
    for diff in diffs {
        println!("  {}: {:?}", diff.path_string, diff.diff_type);
    }
}
