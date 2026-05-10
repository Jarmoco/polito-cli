/* -----------------------------------------------------------------------------
 * tui/notices.rs
 * Notice list viewer: navigate notices, Enter expands content.
 * -------------------------------------------------------------------------- */

use super::Screen;
use crate::{data, display::color, terminal, util};

/* --- Fetch ---------------------------------------------------------------- */

pub fn render(course_id: u64) -> Screen {
    let notices = match data::fetch_notices(course_id) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}.", color::red("ERR"), e);
            println!("{} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Courses;
        }
    };
    let mut selection = 0;
    let mut expanded: Option<usize> = None;

    loop {
        terminal::clear();
        println!("{}", color::bold("  Notices"));
        println!("  {}\n", color::dim("========"));

        if notices.is_empty() {
            println!("  No notices.");
            println!("\n  {} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Courses;
        }

        for (i, n) in notices.iter().enumerate() {
            let id = n["id"].as_i64().unwrap_or(0);
            let date = n["publishedAt"].as_str().unwrap_or("");
            let prefix = if i == selection { ">" } else { " " };

            if expanded == Some(i) {
                println!("  {} {} {}", prefix, color::bold(&format!("#{}", id)), color::dim(date));
                let text = n["content"].as_str().unwrap_or("");
                let clean = util::strip_html(text);
                for line in clean.lines() {
                    println!("    {}", line);
                }
                println!();
            } else {
                println!("  {} {} {}", prefix, color::dim(&format!("#{}", id)), color::dim(date));
            }
        }

        println!("\n  {}", color::dim("Arrows: navigate | Enter: expand | Esc: back"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection < notices.len() - 1 {
                    selection += 1;
                }
            }
            "\r" | "\n" => {
                if expanded == Some(selection) {
                    expanded = None;
                } else {
                    expanded = Some(selection);
                }
            }
            "\x1b" => return Screen::Courses,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}
