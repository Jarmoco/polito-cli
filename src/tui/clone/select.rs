/* -----------------------------------------------------------------------------
 * tui/clone/select.rs
 * File selection screen: tree view with checkboxes, directory-aware toggling,
 * partial state display, bottom summary bar.  Press 'c' to clone with selected
 * files only.
 * -------------------------------------------------------------------------- */

use std::collections::{HashMap, HashSet};
use std::io::Write;

use super::Screen;
use crate::{display::color, terminal, util};
use crate::json::Value;

/* --- Helpers --------------------------------------------------------------- */

fn child_file_paths<'a>(dir_path: &str, files: &'a [(String, Value)]) -> Vec<&'a str> {
    let prefix = format!("{}/", dir_path);
    files
        .iter()
        .filter(|(p, _)| p.starts_with(&prefix))
        .map(|(p, _)| p.as_str())
        .collect()
}

enum DirState {
    All,
    Some,
    None,
}

fn dir_selection_state(
    dir_path: &str,
    selected: &HashSet<String>,
    files: &[(String, Value)],
) -> DirState {
    let children = child_file_paths(dir_path, files);
    if children.is_empty() {
        return DirState::None;
    }
    let sel = children.iter().filter(|p| selected.contains(**p)).count();
    if sel == 0 {
        DirState::None
    } else if sel == children.len() {
        DirState::All
    } else {
        DirState::Some
    }
}

fn selected_bytes(selected: &HashSet<String>, sizes: &HashMap<String, f64>) -> u64 {
    selected
        .iter()
        .filter_map(|p| sizes.get(p))
        .map(|s| *s as u64)
        .sum()
}

/* --- Main Screen ----------------------------------------------------------- */

pub fn run(
    course_id: u64,
    course_name: String,
    shortcode: String,
    entries: Vec<Value>,
    flattened: Vec<(String, Value)>,
) -> Screen {
    let sizes: HashMap<String, f64> = flattened
        .iter()
        .map(|(p, v)| {
            (
                p.clone(),
                v["sizeInKiloBytes"].as_f64().unwrap_or(0.0) * 1024.0,
            )
        })
        .collect();

    let mut selected: HashSet<String> = flattened.iter().map(|(p, _)| p.clone()).collect();
    let mut expanded: HashSet<String> = HashSet::new();
    let mut cursor: usize = 0;
    let mut scroll_off: usize = 0;

    loop {
        let lines = util::build_tree_lines(&entries, &expanded);
        let height = terminal::get_height().max(8);
        let max_visible = height.saturating_sub(6);

        if !lines.is_empty() && cursor >= lines.len() {
            cursor = lines.len() - 1;
        }
        if cursor < scroll_off {
            scroll_off = cursor;
        }
        if !lines.is_empty() && cursor >= scroll_off + max_visible {
            scroll_off = cursor.saturating_sub(max_visible - 1);
        }

        terminal::clear();
        println!("{}", color::bold("  Select Files to Clone"));
        println!("  {}\n", color::dim("======================="));

        if lines.is_empty() {
            println!("  No files.");
            println!("\n  {} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Clone;
        }

        let visible_end = (scroll_off + max_visible).min(lines.len());
        for i in scroll_off..visible_end {
            let tl = &lines[i];
            let indent = "  ".to_string() + &"  ".repeat(tl.depth);
            let prefix = if i == cursor { ">" } else { " " };

            let chk = if tl.is_directory {
                match dir_selection_state(&tl.path, &selected, &flattened) {
                    DirState::All => color::cyan("[x]"),
                    DirState::Some => color::yellow("[-]"),
                    DirState::None => color::dim("[ ]"),
                }
            } else if selected.contains(&tl.path) {
                color::cyan("[x]")
            } else {
                color::dim("[ ]")
            };

            let size_s = if !tl.is_directory && tl.size > 0.0 {
                format!("  {}", color::dim(&terminal::fmt_size(tl.size)))
            } else {
                String::new()
            };

            let line = format!("{}{} {} {}{}", indent, prefix, chk, tl.name, size_s);
            if i == cursor {
                println!("{}", color::cyan(&line));
            } else {
                println!("{}", line);
            }
        }

        if scroll_off > 0 {
            println!(
                "\n  {} {}",
                color::dim("^"),
                color::dim(&format!("{} more", scroll_off))
            );
        }
        let remaining = lines.len().saturating_sub(visible_end);
        if remaining > 0 {
            println!(
                "\n  {} {}",
                color::dim("v"),
                color::dim(&format!("{} more", remaining))
            );
        }

        let sc = selected.len();
        let sb = selected_bytes(&selected, &sizes);
        println!(
            "\n  {} {} file(s), {}   {} {}  {}",
            color::dim("Selected:"),
            sc,
            terminal::fmt_size(sb as f64),
            color::cyan("[c]"),
            color::dim("Clone |"),
            color::dim("[Space] Toggle  [Enter] Expand  [Esc] Back"),
        );
        std::io::stdout().flush().ok();

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if cursor > 0 {
                    cursor -= 1;
                }
            }
            "\x1b[B" => {
                if cursor + 1 < lines.len() {
                    cursor += 1;
                }
            }
            "\r" | "\n" => {
                let tl = &lines[cursor];
                if tl.is_directory {
                    if expanded.contains(&tl.path) {
                        expanded.remove(&tl.path);
                    } else {
                        expanded.insert(tl.path.clone());
                    }
                }
            }
            " " => {
                let tl = &lines[cursor];
                if tl.is_directory {
                    let children = child_file_paths(&tl.path, &flattened);
                    let all = children.iter().all(|p| selected.contains(*p));
                    for p in children {
                        if all {
                            selected.remove(p);
                        } else {
                            selected.insert(p.to_string());
                        }
                    }
                } else if selected.contains(&tl.path) {
                    selected.remove(&tl.path);
                } else {
                    selected.insert(tl.path.clone());
                }
            }
            "c" => {
                if selected.is_empty() {
                    terminal::clear();
                    println!("{} No files selected.", color::yellow("!"));
                    println!("{} Press any key.", color::dim("HINT"));
                    terminal::getch();
                    continue;
                }
                let excluded: HashSet<String> = flattened
                    .iter()
                    .map(|(p, _)| p.clone())
                    .filter(|p| !selected.contains(p))
                    .collect();
                return super::run::run(course_id, course_name, shortcode, excluded);
            }
            "\x1b" => return Screen::Clone,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}
