use clap::Parser;
use compare_json::{compare_json, ArrayCompareMethod, CompareOptions, Difference, PathBelongsTo};
use serde_json::Value;
use std::fs;

#[derive(Parser)]
#[command(name = "compare-json")]
#[command(version)]
#[command(about = "Compare two JSON files or strings")]
struct Cli {
    /// Base JSON string or file path
    base: Option<String>,

    /// Contrast JSON string or file path
    contrast: Option<String>,

    /// Array compare method: byIndex, lcs, unordered
    #[arg(short = 'a', long, default_value = "byIndex")]
    array_compare_method: String,

    /// Case insensitive key comparison
    #[arg(short = 'k', long)]
    key_case_insensitive: bool,

    /// Case insensitive value comparison
    #[arg(short = 'v', long)]
    value_case_insensitive: bool,

    /// Treat numeric strings as numbers
    #[arg(long)]
    numeric_string_equals_number: bool,

    /// Output as JSON format
    #[arg(short = 'j', long)]
    json_export: bool,

    /// Output to file
    #[arg(short = 'o', long)]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let base_str = match &cli.base {
        Some(s) => s.clone(),
        None => {
            Cli::parse_from(["compare-json", "--help"].iter());
            return;
        }
    };

    let contrast_str = match &cli.contrast {
        Some(s) => s.clone(),
        None => {
            Cli::parse_from(["compare-json", "--help"].iter());
            return;
        }
    };

    let base_json = parse_input(&base_str, "base");
    let contrast_json = parse_input(&contrast_str, "contrast");

    let method = match cli.array_compare_method.as_str() {
        "lcs" => Some(ArrayCompareMethod::Lcs),
        "unordered" => Some(ArrayCompareMethod::Unordered),
        _ => Some(ArrayCompareMethod::ByIndex),
    };

    let options = CompareOptions {
        array_compare_method: method,
        key_case_insensitive: cli.key_case_insensitive,
        value_case_insensitive: cli.value_case_insensitive,
        numeric_string_equals_number: cli.numeric_string_equals_number,
    };

    let diffs = compare_json(&base_json, &contrast_json, &options);

    let output = if cli.json_export {
        match serde_json::to_string_pretty(&diffs) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Failed to serialize output. Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        format_table(&diffs)
    };

    if let Some(path) = cli.output {
        if let Err(e) = fs::write(&path, &output) {
            eprintln!("Error: Failed to write output to {}. Error: {}", path, e);
            std::process::exit(1);
        }
        println!("Output written to {}", path);
    } else {
        println!("{}", output);
    }
}

fn parse_input(input: &str, label: &str) -> Value {
    if let Ok(content) = fs::read_to_string(input) {
        match serde_json::from_str(&content) {
            Ok(v) => return v,
            Err(e) => {
                eprintln!(
                    "Error: Failed to parse {} file content from {}, unable to parse as JSON. Error: {}",
                    label, input, e
                );
                std::process::exit(1);
            }
        }
    }
    match serde_json::from_str(input) {
        Ok(v) => v,
        Err(e) => {
            eprintln!(
                "Error: Failed to parse {} input: if you passed a file path, the file was not found; if you passed a JSON string, it failed to parse. Error: {}",
                label, e
            );
            std::process::exit(1);
        }
    }
}

fn format_table(diffs: &[Difference]) -> String {
    if diffs.is_empty() {
        return "No differences found".to_string();
    }

    let rows: Vec<(String, String)> = diffs.iter().map(|d| {
        let key = if d.path_segments.is_empty() {
            "(Root)".to_string()
        } else {
            let prefix = match d.path_belongs_to {
                PathBelongsTo::Contrast => "(Contrast)",
                _ => "(Base)",
            };
            format!("{} {}", prefix, d.path_string)
        };
        (key, d.diff_type.to_string())
    }).collect();

    let max_key = rows.iter().map(|(k, _)| k.len()).max().unwrap().max("Key".len());
    let max_type = rows.iter().map(|(_, t)| t.len()).max().unwrap().max("Change Type".len());

    let mut result = String::new();
    result.push_str(&format!("\u{250c}{}\u{252c}{}\u{2510}\n", "\u{2500}".repeat(max_key + 2), "\u{2500}".repeat(max_type + 2)));
    result.push_str(&format!("\u{2502} {:<width1$} \u{2502} {:<width2$} \u{2502}\n", "Key", "Change Type", width1 = max_key, width2 = max_type));
    result.push_str(&format!("\u{251c}{}\u{253c}{}\u{2524}\n", "\u{2500}".repeat(max_key + 2), "\u{2500}".repeat(max_type + 2)));

    for (key, diff_type) in &rows {
        result.push_str(&format!("\u{2502} {:<width1$} \u{2502} {:<width2$} \u{2502}\n", key, diff_type, width1 = max_key, width2 = max_type));
    }

    result.push_str(&format!("\u{2514}{}\u{2534}{}\u{2518}", "\u{2500}".repeat(max_key + 2), "\u{2500}".repeat(max_type + 2)));
    result
}
