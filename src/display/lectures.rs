/* -----------------------------------------------------------------------------
 * display/lectures.rs
 * Shows lecture timetable with date filters, or outputs JSON.
 * -------------------------------------------------------------------------- */

use crate::{data, display::color, json, util};

/* --- Fetch & Parse -------------------------------------------------------- */

pub fn show_lectures(
    from: Option<&str>,
    to: Option<&str>,
    course_id: Option<u64>,
    json_flag: bool,
) {
    let lectures = match data::fetch_lectures(from, to, course_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    /* --- JSON Output --- */
    if json_flag {
        println!("{}", json::stringify(&json::Value::Array(lectures)));
        return;
    }

    /* --- Table Output --- */
    if lectures.is_empty() {
        println!("{} No lectures.", color::yellow("!"));
        return;
    }

    println!();
    println!(
        "  {:<12} {:<8} {:<6} {:<42} {}",
        color::bold("Date"),
        color::bold("Time"),
        color::bold("Room"),
        color::bold("Course"),
        color::bold("Teacher")
    );
    println!("  {}", color::dim(&"-".repeat(90)));
    for lec in &lectures {
        let starts = lec["startsAt"].as_str().unwrap_or("");
        let ends = lec["endsAt"].as_str().unwrap_or("");
        let date = starts.split('T').next().unwrap_or("-");
        let start = starts.split('T').nth(1).unwrap_or("").split(':').take(2).collect::<Vec<_>>().join(":");
        let end = ends.split('T').nth(1).unwrap_or("").split(':').take(2).collect::<Vec<_>>().join(":");
        let room = lec["place"]["name"].as_str().unwrap_or("-");
        let course = lec["courseName"].as_str().unwrap_or("-");
        let teacher_id = lec["teacherId"].as_i64().unwrap_or(0);
        let teacher = if teacher_id > 0 { teacher_id.to_string() } else { "-".to_string() };
        println!(
            "  {:<12} {}-{} {:<6} {:<42} {}",
            date,
            start,
            end,
            room,
            util::trunc(course, 40),
            color::dim(&teacher)
        );
    }
    println!();
}
