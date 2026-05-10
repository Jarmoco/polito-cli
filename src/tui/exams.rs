/* -----------------------------------------------------------------------------
 * tui/exams.rs
 * Exam table viewer: navigate exams, color-coded status.
 * -------------------------------------------------------------------------- */

use super::Screen;
use crate::{data, display::color, terminal, util};

/* --- Fetch ---------------------------------------------------------------- */

pub fn render() -> Screen {
    let exams = match data::fetch_exams() {
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
        println!("{}", color::bold("  Exams"));
        println!("  {}\n", color::dim("======"));

        if exams.is_empty() {
            println!("  No exams.");
            println!("\n  {} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }

        println!(
            "  {:<28} {:<12} {:<12} {:<6}",
            color::bold("Course"),
            color::bold("Date"),
            color::bold("Status"),
            color::bold("Grade")
        );
        println!("  {}", color::dim(&"-".repeat(62)));

        for (i, e) in exams.iter().enumerate() {
            let course = e["courseName"].as_str().unwrap_or("-");
            let date = e["examStartsAt"].as_str().unwrap_or("-");
            let date = date.split('T').next().unwrap_or(date);
            let status = e["status"].as_str().unwrap_or("");
            let grade = "-";
            let status_color = match status {
                "booked" => color::green("booked"),
                "available" => color::yellow("available"),
                "unavailable" => color::red("unavailable"),
                "requestable" => color::cyan("requestable"),
                "requested" => color::dim("requested"),
                _ => color::dim(status),
            };
            let prefix = if i == selection { ">" } else { " " };
            println!("{} {:<28} {:<12} {:<12} {:<6}", prefix, util::trunc(course, 26), date, status_color, grade);
        }

        println!("\n  {}", color::dim("Arrows: navigate | Esc: back"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection < exams.len() - 1 {
                    selection += 1;
                }
            }
            "\x1b" => return Screen::Menu,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}
