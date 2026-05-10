/* -----------------------------------------------------------------------------
 * tui/clone/mod.rs
 * Course selection screen → dispatches to select::run for file selection.
 * -------------------------------------------------------------------------- */

use super::Screen;
use crate::{data, display::color, terminal, util};

mod run;
mod select;

/* --- Render ---------------------------------------------------------------- */

pub fn render() -> Screen {
    let courses = match data::fetch_courses() {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}.", color::red("ERR"), e);
            println!("{} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }
    };
    let mut selection: usize = 0;

    loop {
        terminal::clear();
        println!("{}", color::bold("  Clone Course"));
        println!("  {}\n", color::dim("============="));
        println!("  {} Select a course to clone:\n", color::dim("HINT"));

        if courses.is_empty() {
            println!("  No courses.");
            terminal::getch();
            return Screen::Menu;
        }

        for (i, c) in courses.iter().enumerate() {
            let name = c["name"].as_str().unwrap_or("-");
            let code = c["shortcode"].as_str().unwrap_or("");
            if i == selection {
                println!("  > {} {}  {}", color::cyan(name), color::yellow(code), color::dim("[Enter to clone]"));
            } else {
                println!("    {} {}", name, code);
            }
        }

        println!("\n  {}", color::dim("Arrows: navigate | Enter: clone | Esc: back"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection + 1 < courses.len() {
                    selection += 1;
                }
            }
            "\r" | "\n" => {
                let cid = courses[selection]["id"].as_i64().unwrap_or(0) as u64;
                let name = courses[selection]["name"].as_str().unwrap_or("unknown").to_string();
                let code = courses[selection]["shortcode"].as_str().unwrap_or("").to_string();

                let entries = match data::fetch_files(cid) {
                    Ok(e) => e,
                    Err(e) => {
                        terminal::clear();
                        println!("{} {}.", color::red("ERR"), e);
                        println!("{} Press any key.", color::dim("HINT"));
                        terminal::getch();
                        return Screen::Menu;
                    }
                };
                let flattened: Vec<_> = util::flatten_files(&entries, "")
                    .into_iter()
                    .map(|(p, v)| (p, v.clone()))
                    .collect();

                if flattened.is_empty() {
                    terminal::clear();
                    println!("{} No files to clone.", color::yellow("!"));
                    println!("{} Press any key.", color::dim("HINT"));
                    terminal::getch();
                    return Screen::Menu;
                }

                let screen = select::run(cid, name, code, entries, flattened);
                if screen != Screen::Clone {
                    return screen;
                }
            }
            "\x1b" => return Screen::Menu,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}
