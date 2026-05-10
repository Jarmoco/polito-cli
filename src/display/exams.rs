/* -----------------------------------------------------------------------------
 * display/exams.rs
 * Lists upcoming exams with booking status coloring, or outputs JSON.
 * -------------------------------------------------------------------------- */

use crate::{data, display::color, json};

/* --- Fetch ---------------------------------------------------------------- */

pub fn show_exams(json_flag: bool) {
    let exams = match data::fetch_exams() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    /* --- JSON Output --- */
    if json_flag {
        println!("{}", json::stringify(&json::Value::Array(exams)));
        return;
    }

    /* --- Table Output --- */
    if exams.is_empty() {
        println!("{} No exams.", color::yellow("!"));
        return;
    }

    println!();
    println!(
        "  {:<30} {:<12} {:<10} {}",
        color::bold("Course"),
        color::bold("Date"),
        color::bold("Status"),
        color::bold("Grade")
    );
    println!("  {}", color::dim(&"-".repeat(70)));
    for exam in &exams {
        let course = exam["courseName"].as_str().unwrap_or("-");
        let date = exam["examStartsAt"].as_str().unwrap_or("-");
        let date = date.split('T').next().unwrap_or(date);
        let status = exam["status"].as_str().unwrap_or("");
        let grade = "-";
        let status_colored = match status {
            "booked" => color::green("booked"),
            "available" => color::yellow("available"),
            "unavailable" => color::red("unavailable"),
            "requestable" => color::cyan("requestable"),
            "requested" => color::dim("requested"),
            _ => color::dim(status),
        };
        println!("  {:<30} {:<12} {:<10} {}", course, date, status_colored, grade);
    }
    println!();
}


