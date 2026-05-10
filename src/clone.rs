/* -----------------------------------------------------------------------------
 * clone.rs
 * Course clone orchestrator: total size, progress callback, resume from partial
 * metadata, incremental save after each file download.
 * -------------------------------------------------------------------------- */

use std::collections::HashSet;

use crate::{conflict, data, display::color, download, json, meta, util};

/* --- Types ----------------------------------------------------------------- */

pub struct ProgressInfo {
    pub files_done: usize,
    pub files_total: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
    pub current_file: String,
}

pub struct CloneOptions {
    pub course_id: u64,
    pub course_name: Option<String>,
    pub shortcode: Option<String>,
    pub output_dir: Option<String>,
    pub overwrite: bool,
    pub skip_existing: bool,
    pub backup: bool,
    pub progress_cb: Option<Box<dyn FnMut(&ProgressInfo)>>,
    pub excluded_paths: Option<HashSet<String>>,
}

/* --- Helpers --------------------------------------------------------------- */

pub fn calculate_total_size(flattened_files: &[(String, json::Value)]) -> u64 {
    let mut total_kb: f64 = 0.0;
    for (_, f) in flattened_files {
        total_kb += f["sizeInKiloBytes"].as_f64().unwrap_or(0.0);
    }
    (total_kb * 1024.0) as u64
}

pub fn get_course_size(course_id: u64) -> Result<(usize, u64), String> {
    let entries = data::fetch_files(course_id)?;
    let flat = util::flatten_files(&entries, "");
    let count = flat.len();
    let owned: Vec<_> = flat.into_iter().map(|(p, v)| (p, v.clone())).collect();
    Ok((count, calculate_total_size(&owned)))
}

fn fetch_course_info(course_id: u64, opts: &CloneOptions) -> (String, String) {
    if let (Some(n), Some(c)) = (&opts.course_name, &opts.shortcode) {
        return (n.clone(), c.clone());
    }
    let courses = data::fetch_courses().unwrap_or_else(|e| {
        eprintln!("{} {}", color::red("x"), e);
        std::process::exit(1);
    });
    match courses.iter().find(|c| c["id"].as_i64() == Some(course_id as i64)) {
        Some(c) => (c["name"].as_str().unwrap_or("?").to_string(), c["shortcode"].as_str().unwrap_or("").to_string()),
        None => { eprintln!("{} Course {} not found.", color::red("x"), course_id); std::process::exit(1); }
    }
}

fn fetch_file_entries(course_id: u64) -> Vec<(String, json::Value)> {
    let entries = data::fetch_files(course_id).unwrap_or_else(|e| {
        eprintln!("{} {}", color::red("x"), e);
        std::process::exit(1);
    });
    util::flatten_files(&entries, "").into_iter().map(|(p, v)| (p, v.clone())).collect()
}

/* --- Download Loop --------------------------------------------------------- */

fn download_all_files(
    flattened_files: &[(String, json::Value)],
    course_id: u64,
    out_dir: &str,
    skip_existing: bool,
    backup: bool,
    progress_cb: &mut Option<Box<dyn FnMut(&ProgressInfo)>>,
    existing_meta: &Option<meta::CloneMetadata>,
    meta_path: &str,
) -> meta::CloneMetadata {
    let total_bytes = calculate_total_size(flattened_files);
    let total = flattened_files.len();
    let mut metadata = meta::CloneMetadata {
        course_id,
        course_name: String::new(),
        shortcode: String::new(),
        files: Vec::with_capacity(total),
    };
    let completed: HashSet<String> = existing_meta
        .as_ref()
        .map(|m| m.files.iter().map(|f| f.path.clone()).collect())
        .unwrap_or_default();

    let mut bytes_done: u64 = 0;
    let mut failed: Vec<String> = Vec::new();

    for (rel_path, file) in flattened_files {
        let file_id = file["id"].as_str().unwrap_or("").to_string();
        let checksum = file["checksum"].as_str().map(String::from);
        let size_bytes = (file["sizeInKiloBytes"].as_f64().unwrap_or(0.0) * 1024.0) as u64;
        let dest = format!("{}/{}", out_dir, rel_path);
        let on_disk = std::path::Path::new(&dest).exists();
        let skip = on_disk && (completed.contains(rel_path) || skip_existing);

        if skip {
            bytes_done += size_bytes;
            metadata.files.push(meta::FileEntry {
                path: rel_path.clone(),
                id: file_id,
                checksum,
            });
        } else {
            if on_disk && backup {
                let _ = std::fs::rename(&dest, format!("{}.bak", dest));
            }
            if let Some(p) = std::path::Path::new(&dest).parent() {
                std::fs::create_dir_all(p).unwrap_or_else(|e| {
                    eprintln!("{} Failed to create dir {}: {}", color::red("x"), p.display(), e);
                    std::process::exit(1);
                });
            }
            let ok = match download::download_to_path(course_id, &file_id, &dest) {
                Ok(s) => (200..300).contains(&s),
                Err(_) => false,
            };
            if ok {
                bytes_done += size_bytes;
            } else {
                failed.push(rel_path.clone());
            }
            metadata.files.push(meta::FileEntry {
                path: rel_path.clone(),
                id: file_id,
                checksum: if ok { checksum } else { None },
            });
            if ok {
                meta::save_metadata(meta_path, &metadata);
            }
        }

        if let Some(ref mut cb) = progress_cb {
            cb(&ProgressInfo {
                files_done: metadata.files.len(),
                files_total: total,
                bytes_done,
                bytes_total: total_bytes,
                current_file: rel_path.clone(),
            });
        }
    }

    if !failed.is_empty() {
        eprintln!("\n{} {} file(s) failed:", color::red("!"), failed.len());
        for f in &failed {
            eprintln!("  - {}", f);
        }
        eprintln!("\n{} Partial clone saved to {}", color::yellow("!"), out_dir);
    }

    let success_count = total - failed.len();
    println!("{} Cloned {} file(s) into {}", color::green("v"), success_count, out_dir);
    metadata
}

/* --- Main ----------------------------------------------------------------- */

pub fn clone_course(opts: CloneOptions) {
    let (course_name, shortcode) = fetch_course_info(opts.course_id, &opts);
    let out_dir = opts.output_dir.unwrap_or_else(|| {
        format!("polito-{}", course_name.replace(' ', "-").to_lowercase())
    });
    std::fs::create_dir_all(&out_dir).unwrap_or_else(|e| {
        eprintln!("{} Failed to create output directory: {}", color::red("x"), e);
        std::process::exit(1);
    });

    let meta_path = format!("{}/.polito-clone.json", out_dir);
    let existing_metadata = meta::load_metadata(&meta_path);
    let flattened_files = fetch_file_entries(opts.course_id);
    let flattened_files = if let Some(ref excluded) = opts.excluded_paths {
        flattened_files
            .into_iter()
            .filter(|(p, _)| !excluded.contains(p))
            .collect()
    } else {
        flattened_files
    };
    let mut progress_cb = opts.progress_cb;

    conflict::detect_conflicts(&flattened_files, &existing_metadata, opts.overwrite, opts.skip_existing, opts.backup);

    let mut metadata = download_all_files(&flattened_files, opts.course_id, &out_dir, opts.skip_existing, opts.backup, &mut progress_cb, &existing_metadata, &meta_path);

    metadata.course_name = course_name;
    metadata.shortcode = shortcode;
    meta::save_metadata(&meta_path, &metadata);
}
