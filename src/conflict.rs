/* -----------------------------------------------------------------------------
 * conflict.rs
 * Conflict detection: compare server checksums against local metadata to
 * classify each file as changed, added, or removed since the last clone.
 * -------------------------------------------------------------------------- */

use crate::{display::color, json, meta};

pub fn detect_conflicts(
    flattened_files: &[(String, json::Value)],
    existing: &Option<meta::CloneMetadata>,
    overwrite: bool,
    skip_existing: bool,
    backup: bool,
) {
    let metadata = match existing {
        Some(m) => m,
        None => return,
    };

    let mut conflicts: Vec<(String, String)> = Vec::new();
    for (path, server_file) in flattened_files {
        let found = metadata.files.iter().find(|f| f.path == *path);
        match found {
            Some(local) => {
                if local.checksum.as_deref() != server_file["checksum"].as_str() {
                    conflicts.push(("changed".into(), path.clone()));
                }
            }
            None => conflicts.push(("added".into(), path.clone())),
        }
    }
    for local in &metadata.files {
        if !flattened_files.iter().any(|(p, _)| *p == local.path) {
            conflicts.push(("removed".into(), local.path.clone()));
        }
    }

    if !conflicts.is_empty() && !overwrite && !skip_existing && !backup {
        eprintln!("{} {} conflict(s) detected:", color::yellow("!"), conflicts.len());
        for (kind, path) in &conflicts {
            eprintln!("  [{}] {}", kind, path);
        }
        eprintln!();
        eprintln!("Re-run with one of: --overwrite, --skip-existing, --backup");
        std::process::exit(1);
    }
}
