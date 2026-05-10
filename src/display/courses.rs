/* -----------------------------------------------------------------------------
 * display/courses.rs
 * Lists enrolled courses in a table or outputs JSON.
 * -------------------------------------------------------------------------- */

use crate::{data, display::color, json, util};

/* --- Fetch & Parse -------------------------------------------------------- */

pub fn show_courses(year_filter: Option<&str>, json_flag: bool) {
    let courses = match data::fetch_courses() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };
    let filtered: Vec<_> = if let Some(year) = year_filter {
        let mut result = Vec::new();
        for c in courses.iter() {
            if c["year"].as_str() == Some(year) {
                result.push(c);
            }
        }
        result
    } else {
        courses.iter().collect()
    };

    if json_flag {
        let mut arr: Vec<json::Value> = Vec::new();
        for c in filtered.iter() {
            arr.push((*c).clone());
        }
        println!("{}", json::stringify(&json::Value::Array(arr)));
        return;
    }

    /* --- Table Output --- */
    if filtered.is_empty() {
        println!("{} No courses found.", color::yellow("!"));
        return;
    }

    println!();
    println!(
        "  {:<10} {:<10} {:<42} {:<20} {}",
        color::bold("ID"),
        color::bold("Code"),
        color::bold("Name"),
        color::bold("Teacher"),
        color::bold("Year")
    );
    println!("  {}", color::dim(&"-".repeat(92)));
    for course in &filtered {
        let id = course["id"].as_i64().unwrap_or(0);
        let code = course["shortcode"].as_str().unwrap_or("-");
        let name = course["name"].as_str().unwrap_or("-");
        let teacher = course["teacherName"].as_str().unwrap_or("-");
        let year = course["year"].as_str().unwrap_or("-");
        println!(
            "  {:<10} {:<10} {:<42} {:<20} {}",
            color::cyan(&id.to_string()),
            color::yellow(code),
            util::trunc(name, 40),
            color::dim(util::trunc(teacher, 18).as_str()),
            color::dim(year)
        );
    }
    println!();
    println!(
        "  {} course(s). Run `polito files <ID>` to browse materials.",
        color::bold(&filtered.len().to_string())
    );
    println!();
}
