/* -----------------------------------------------------------------------------
 * tui/courses.rs
 * Course list screen: fetch courses, navigate, Enter drills into files/notices.
 * -------------------------------------------------------------------------- */

use super::Screen;
use crate::{data, display::color, terminal};

/* --- Fetch ---------------------------------------------------------------- */

pub fn render() -> Screen {
    let courses = match data::fetch_courses() {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}.", color::red("ERR"), e);
            println!("{} Press any key to go back.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }
    };
    let mut selection = 0;

    loop {
        terminal::clear();
        println!("{}", color::bold("  Courses"));
        println!("  {}\n", color::dim("========"));

        if courses.is_empty() {
            println!("  No courses found.");
            println!("\n  {} Press any key to go back.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }

        for (i, c) in courses.iter().enumerate() {
            let name = c["name"].as_str().unwrap_or("-");
            let code = c["shortcode"].as_str().unwrap_or("");
            let year = c["year"].as_str().unwrap_or("");
            if i == selection {
                println!("  > {} {} {} ({})", color::cyan(name), color::yellow(code), color::dim(year), color::dim("Enter: files, n: notices"));
            } else {
                println!("    {} {} ({})", name, code, year);
            }
        }

        println!("\n  {}", color::dim("Arrows: navigate | Enter: files | n: notices | Esc: back"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection < courses.len() - 1 {
                    selection += 1;
                }
            }
            "\r" | "\n" => {
                let cid = courses[selection]["id"].as_i64().unwrap_or(0) as u64;
                return Screen::Files(cid);
            }
            "n" => {
                let cid = courses[selection]["id"].as_i64().unwrap_or(0) as u64;
                return Screen::Notices(cid);
            }
            "\x1b" => return Screen::Menu,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}
