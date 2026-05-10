/* -----------------------------------------------------------------------------
 * display/grades.rs
 * Shows recorded grades (handles int and "30L" string grades), or outputs JSON.
 * -------------------------------------------------------------------------- */

use crate::{data, display::color, json};

/* --- Fetch ---------------------------------------------------------------- */

pub fn show_grades(json_flag: bool) {
    let grades = match data::fetch_grades() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{}", e);
            return;
        }
    };

    /* --- JSON Output --- */
    if json_flag {
        println!("{}", json::stringify(&json::Value::Array(grades)));
        return;
    }

    /* --- Table Output --- */
    if grades.is_empty() {
        println!("{} No grades.", color::yellow("!"));
        return;
    }

    println!();
    println!(
        "  {:<42} {:<8} {:<10} {}",
        color::bold("Course"),
        color::bold("Grade"),
        color::bold("Date"),
        color::bold("Type")
    );
    println!("  {}", color::dim(&"-".repeat(72)));
    for g in &grades {
        let course = g["courseName"].as_str().unwrap_or("-");
        let grade = format_grade(g);
        let date = g["date"].as_str().unwrap_or("-");
        let exam_type = g["examType"].as_str().unwrap_or("-");
        let grade_colored = color_grade(&grade);
        println!(
            "  {:<42} {:<8} {:<10} {}",
            course, grade_colored, date, exam_type
        );
    }
    println!();
}

/* --- Helpers -------------------------------------------------------------- */

fn format_grade(g: &json::Value) -> String {
    if let Some(n) = g["grade"].as_i64() {
        n.to_string()
    } else if let Some(s) = g["grade"].as_str() {
        s.to_string()
    } else {
        "-".to_string()
    }
}

fn color_grade(grade: &str) -> String {
    if grade == "30L" {
        return color::green(grade);
    }
    if let Ok(n) = grade.parse::<f64>() {
        if n >= 28.0 {
            return color::green(grade);
        } else if n >= 24.0 {
            return color::yellow(grade);
        }
    }
    color::dim(grade)
}
