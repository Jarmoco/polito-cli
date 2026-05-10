/* -----------------------------------------------------------------------------
 * data.rs
 * Shared data access layer: fetch functions for courses, files, notices,
 * exams, lectures, grades. Used by both CLI (display/) and TUI (tui/).
 * -------------------------------------------------------------------------- */

use crate::{api, json};

/* --- Internal Helper ------------------------------------------------------ */

fn fetch_data(path: &str) -> Result<Vec<json::Value>, String> {
    let resp = api::get(path).map_err(|e| format!("Network error: {}", e))?;
    let status = resp.status;
    let body = resp.body;
    if status != 200 {
        return Err(format!("HTTP {}", status));
    }
    let parsed = json::parse(&body).map_err(|e| format!("Response error: {}", e))?;
    Ok(parsed["data"].as_array().map(|a| a.to_vec()).unwrap_or_default())
}

/* --- Courses --------------------------------------------------------------- */

pub fn fetch_courses() -> Result<Vec<json::Value>, String> {
    fetch_data("/v2/courses")
}

/* --- Files ----------------------------------------------------------------- */

pub fn fetch_files(course_id: u64) -> Result<Vec<json::Value>, String> {
    fetch_data(&format!("/courses/{}/files", course_id))
}

/* --- Notices --------------------------------------------------------------- */

pub fn fetch_notices(course_id: u64) -> Result<Vec<json::Value>, String> {
    fetch_data(&format!("/courses/{}/notices", course_id))
}

/* --- Exams ----------------------------------------------------------------- */

pub fn fetch_exams() -> Result<Vec<json::Value>, String> {
    fetch_data("/exams")
}

/* --- Lectures -------------------------------------------------------------- */

pub fn fetch_lectures(
    from: Option<&str>,
    to: Option<&str>,
    course_id: Option<u64>,
) -> Result<Vec<json::Value>, String> {
    let mut path = "/lectures".to_string();
    let mut params: Vec<String> = Vec::new();
    if let Some(f) = from {
        params.push(format!("fromDate={}", f));
    }
    if let Some(t) = to {
        params.push(format!("toDate={}", t));
    }
    if let Some(cid) = course_id {
        params.push(format!("courseIds[]={}", cid));
    }
    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }
    fetch_data(&path)
}

/* --- Grades ---------------------------------------------------------------- */

pub fn fetch_grades() -> Result<Vec<json::Value>, String> {
    fetch_data("/grades")
}
