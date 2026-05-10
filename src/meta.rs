/* -----------------------------------------------------------------------------
 * meta.rs
 * Clone metadata: file checksums, progress checkpoint, load and save from JSON.
 * -------------------------------------------------------------------------- */

use crate::json;

/* --- Types ----------------------------------------------------------------- */

pub struct CloneMetadata {
    pub course_id: u64,
    pub course_name: String,
    pub shortcode: String,
    pub files: Vec<FileEntry>,
}

pub struct FileEntry {
    pub path: String,
    pub id: String,
    pub checksum: Option<String>,
}

/* --- Load ------------------------------------------------------------------ */

pub fn load_metadata(path: &str) -> Option<CloneMetadata> {
    if !std::path::Path::new(path).exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    let v = json::parse(&content).ok()?;
    let files_array = v["files"].as_array()?;
    let mut files: Vec<FileEntry> = Vec::new();
    for f in files_array {
        let path = match f["path"].as_str() {
            Some(p) => p.to_string(),
            None => continue,
        };
        let id = match f["id"].as_str() {
            Some(i) => i.to_string(),
            None => continue,
        };
        files.push(FileEntry {
            path,
            id,
            checksum: f["checksum"].as_str().map(String::from),
        });
    }
    Some(CloneMetadata {
        course_id: v["course_id"].as_i64()? as u64,
        course_name: v["course_name"].as_str()?.to_string(),
        shortcode: v["shortcode"].as_str().unwrap_or("").to_string(),
        files,
    })
}

/* --- Save ------------------------------------------------------------------ */

pub fn save_metadata(path: &str, meta: &CloneMetadata) {
    let mut files_json: Vec<json::Value> = Vec::new();
    for f in &meta.files {
        let mut map = std::collections::HashMap::new();
        map.insert("path".to_string(), json::Value::String(f.path.clone()));
        map.insert("id".to_string(), json::Value::String(f.id.clone()));
        map.insert(
            "checksum".to_string(),
            f.checksum
                .clone()
                .map(json::Value::String)
                .unwrap_or(json::Value::Null),
        );
        files_json.push(json::Value::Object(map));
    }

    let mut map = std::collections::HashMap::new();
    map.insert(
        "course_id".to_string(),
        json::Value::Number(meta.course_id as f64),
    );
    map.insert(
        "course_name".to_string(),
        json::Value::String(meta.course_name.clone()),
    );
    map.insert(
        "shortcode".to_string(),
        json::Value::String(meta.shortcode.clone()),
    );
    map.insert("files".to_string(), json::Value::Array(files_json));
    map.insert(
        "cloned_at".to_string(),
        json::Value::String(crate::util::utc_timestamp()),
    );

    let v = json::Value::Object(map);
    if let Err(e) = std::fs::write(path, json::stringify(&v)) {
        eprintln!("  warn: failed to save metadata: {}", e);
    }
}
