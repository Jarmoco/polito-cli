/* -----------------------------------------------------------------------------
 * util/mod.rs
 * Shared helpers: file flattening, tree building, plus re-exports from
 * sub-modules for HTML, strings, filesystem, and date operations.
 * -------------------------------------------------------------------------- */

pub mod date;
pub mod fs;
pub mod html;
pub mod string;

pub use date::{format_week_label, get_week_bounds, utc_timestamp};
pub use fs::{open_file, pick_directory};
pub use html::strip_html;
pub use string::trunc;

use std::collections::HashSet;

use crate::json::Value;

/* --- File Flattening ------------------------------------------------------ */

pub fn flatten_files<'a>(entries: &'a [Value], prefix: &str) -> Vec<(String, &'a Value)> {
    let mut result = Vec::new();
    for entry in entries {
        let entry_type = entry["type"].as_str();
        let name = entry["name"].as_str().unwrap_or("");
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", prefix, name)
        };
        match entry_type {
            Some("directory") => {
                if let Some(files) = entry["files"].as_array() {
                    result.extend(flatten_files(files, &path));
                }
            }
            Some("file") => {
                result.push((path, entry));
            }
            _ => {}
        }
    }
    result
}

/* --- Tree Walk ------------------------------------------------------------ */

#[derive(Debug, Clone)]
pub struct TreeLine {
    pub path: String,
    pub name: String,
    pub id: Option<String>,
    pub depth: usize,
    pub is_directory: bool,
    pub size: f64,
}

pub fn build_tree_lines(entries: &[Value], expanded: &HashSet<String>) -> Vec<TreeLine> {
    let mut lines = Vec::new();
    walk_tree(entries, 0, "", expanded, &mut lines);
    lines
}

fn walk_tree(
    entries: &[Value],
    depth: usize,
    prefix: &str,
    expanded: &HashSet<String>,
    out: &mut Vec<TreeLine>,
) {
    for entry in entries {
        let entry_type = entry["type"].as_str();
        let name = entry["name"].as_str().unwrap_or("");
        if name.is_empty() {
            continue;
        }
        let path = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", prefix, name)
        };
        let is_dir = entry_type == Some("directory");
        let size = entry["sizeInKiloBytes"].as_f64().unwrap_or(0.0) * 1024.0;

        out.push(TreeLine {
            path: path.clone(),
            name: name.to_string(),
            id: entry["id"].as_str().map(String::from),
            depth,
            is_directory: is_dir,
            size,
        });

        if is_dir && expanded.contains(&path) {
            if let Some(files) = entry["files"].as_array() {
                walk_tree(files, depth + 1, &path, expanded, out);
            }
        }
    }
}
