/* -----------------------------------------------------------------------------
 * display/files.rs
 * Displays course file tree (recursive directory walk) or outputs JSON.
 * -------------------------------------------------------------------------- */

use crate::{data, display::color, json, terminal, util};

/* --- Fetch & Parse -------------------------------------------------------- */

pub fn show_files(course_id: u64, path_filter: Option<&str>, json_flag: bool) {
    let entries = match data::fetch_files(course_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let flat = util::flatten_files(&entries, "");

    /* --- JSON Output --- */
    if json_flag {
        let mut files: Vec<json::Value> = Vec::new();
        for pair in &flat {
            let (p, f) = pair;
            let mut map = std::collections::HashMap::new();
            map.insert("path".to_string(), json::Value::String(p.clone()));
            map.insert(
                "name".to_string(),
                json::Value::String(f["name"].as_str().unwrap_or("").to_string()),
            );
            map.insert(
                "size".to_string(),
                json::Value::Number(
                    f["sizeInKiloBytes"].as_f64().unwrap_or(0.0) * 1024.0,
                ),
            );
            files.push(json::Value::Object(map));
        }
        println!("{}", json::stringify(&json::Value::Array(files)));
        return;
    }

    let filtered: Vec<&(String, &json::Value)> = if let Some(filter) = path_filter {
        let mut result = Vec::new();
        for pair in flat.iter() {
            if pair.0.contains(filter) {
                result.push(pair);
            }
        }
        result
    } else {
        flat.iter().collect()
    };

    if filtered.is_empty() {
        println!("{} No files found.", color::yellow("!"));
        return;
    }

    println!();
    for pair in &filtered {
        let (path, file) = *pair;
        let ftype = file["type"].as_str().unwrap_or("file");
        let size = file["sizeInKiloBytes"].as_f64().unwrap_or(0.0) * 1024.0;
        let name = file["name"].as_str().unwrap_or("-");
        let id = file["id"].as_str().unwrap_or("-");
        if ftype == "directory" {
            println!("  {} {} {}", color::cyan("[/]"), name, color::dim(&terminal::fmt_size(size)));
        } else {
            println!(
                "  {} {}  {}  ({})",
                color::dim("[f]"),
                util::trunc(path, 50),
                color::dim(&terminal::fmt_size(size)),
                color::dim(id)
            );
        }
    }
    println!();
    println!(
        "  {} file(s). Use --path <ID> to download.",
        color::bold(&filtered.len().to_string())
    );
    println!();
}
