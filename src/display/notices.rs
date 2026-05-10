/* -----------------------------------------------------------------------------
 * display/notices.rs
 * Lists course notices with HTML stripped, or outputs JSON.
 * -------------------------------------------------------------------------- */

use crate::{data, display::color, json, util};

/* --- Fetch ---------------------------------------------------------------- */

pub fn show_notices(course_id: u64, json_flag: bool) {
    let notices = match data::fetch_notices(course_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    /* --- JSON Output --- */
    if json_flag {
        println!("{}", json::stringify(&json::Value::Array(notices)));
        return;
    }

    /* --- Text Output --- */
    if notices.is_empty() {
        println!("{} No notices.", color::yellow("!"));
        return;
    }

    println!();
    for notice in notices.iter() {
        let id = notice["id"].as_i64().unwrap_or(0) as u64;
        let date = notice["publishedAt"].as_str().unwrap_or("");
        let text = notice["content"].as_str().unwrap_or("");

        println!("  {} {}", color::bold(&format!("#{}", id)), color::dim(date));
        if !text.is_empty() {
            println!("  {}", util::trunc(&util::strip_html(text), 200));
        }
        println!();
    }
}
