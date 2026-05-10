/* -----------------------------------------------------------------------------
 * tui/grades.rs
 * Grade table viewer: color-coded (green >= 28, yellow >= 24, dim otherwise).
 * -------------------------------------------------------------------------- */

use super::Screen;
use crate::{data, display::color, json, terminal, util};

/* --- Fetch ---------------------------------------------------------------- */

pub fn render() -> Screen {
    let grades = match data::fetch_grades() {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}.", color::red("ERR"), e);
            println!("{} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }
    };
    let mut selection = 0;

    loop {
        terminal::clear();
        println!("{}", color::bold("  Grades"));
        println!("  {}\n", color::dim("======="));

        if grades.is_empty() {
            println!("  No grades.");
            println!("\n  {} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }

        println!(
            "  {:<38} {:<8} {:<10} {:<10}",
            color::bold("Course"),
            color::bold("Grade"),
            color::bold("Date"),
            color::bold("Type")
        );
        println!("  {}", color::dim(&"-".repeat(70)));

        for (i, g) in grades.iter().enumerate() {
            let course = util::trunc(g["courseName"].as_str().unwrap_or("-"), 36);
            let grade_str = format_grade(g);
            let date = g["date"].as_str().unwrap_or("-");
            let exam_type = g["shortcode"].as_str().unwrap_or("-");
            let gc = color_grade(&grade_str);
            let prefix = if i == selection { ">" } else { " " };
            println!("{} {:<38} {:<8} {:<10} {:<10}", prefix, course, gc, date, exam_type);
        }

        println!("\n  {}", color::dim("Arrows: navigate | Esc: back"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection < grades.len() - 1 {
                    selection += 1;
                }
            }
            "\x1b" => return Screen::Menu,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
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
