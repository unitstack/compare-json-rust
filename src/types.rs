use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffType {
    Added,
    Deleted,
    TypeChanged,
    ValueChanged,
}

impl std::fmt::Display for DiffType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DiffType::Added => write!(f, "added"),
            DiffType::Deleted => write!(f, "deleted"),
            DiffType::TypeChanged => write!(f, "typeChanged"),
            DiffType::ValueChanged => write!(f, "valueChanged"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PathBelongsTo {
    Base,
    Contrast,
    Both,
}

impl std::fmt::Display for PathBelongsTo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathBelongsTo::Base => write!(f, "base"),
            PathBelongsTo::Contrast => write!(f, "contrast"),
            PathBelongsTo::Both => write!(f, "both"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Difference {
    pub path_segments: Vec<String>,
    pub path_string: String,
    pub path_belongs_to: PathBelongsTo,
    pub diff_type: DiffType,
}

#[derive(Debug, Clone, Copy)]
pub enum ArrayCompareMethod {
    ByIndex,
    Lcs,
    Unordered,
}

#[derive(Debug, Clone, Default)]
pub struct CompareOptions {
    pub array_compare_method: Option<ArrayCompareMethod>,
    pub key_case_insensitive: bool,
    pub value_case_insensitive: bool,
    pub numeric_string_equals_number: bool,
}
