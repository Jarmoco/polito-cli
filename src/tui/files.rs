/* -----------------------------------------------------------------------------
 * tui/files.rs
 * Tree-based file browser: expand/collapse directories, scrollable viewport,
 * arrow navigation, scroll markers when content overflows terminal.
 * -------------------------------------------------------------------------- */

use std::collections::HashSet;
use std::io::Write;

use super::Screen;
use crate::{data, display::color, download, terminal, util};

/* --- Fetch ---------------------------------------------------------------- */

pub fn render(course_id: u64) -> Screen {
    let entries = match data::fetch_files(course_id) {
        Ok(c) => c,
        Err(e) => {
            println!("{} {}.", color::red("ERR"), e);
            println!("{} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Courses;
        }
    };

    let mut expanded: HashSet<String> = HashSet::new();
    let mut selection: usize = 0;
    let mut scroll_offset: usize = 0;

    loop {
        let lines = util::build_tree_lines(&entries, &expanded);
        let height = terminal::get_height().max(6);
        let max_visible = height - 4;

        if lines.is_empty() {
            terminal::clear();
            println!("{}", color::bold("  Files"));
            println!("  {}\n", color::dim("======"));
            println!("  No files.");
            println!("\n  {} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Courses;
        }

        if selection >= lines.len() {
            selection = lines.len() - 1;
        }

        if selection < scroll_offset {
            scroll_offset = selection;
        }
        if selection >= scroll_offset + max_visible {
            scroll_offset = selection.saturating_sub(max_visible - 1);
        }

        terminal::clear();
        println!("{}", color::bold("  Files"));
        println!("  {}\n", color::dim("======"));

        if scroll_offset > 0 {
            println!("  {} {}\n", color::dim("^"), color::dim(&format!("{} more", scroll_offset)));
        }

        let visible_end = (scroll_offset + max_visible).min(lines.len());
        for i in scroll_offset..visible_end {
            let tree_line = &lines[i];
            let indent = "  ".to_string() + &"  ".repeat(tree_line.depth);

            let prefix = if i == selection { ">" } else { " " };

            let line = if tree_line.is_directory {
                let icon = if expanded.contains(&tree_line.path) {
                    color::cyan("[-]")
                } else {
                    color::yellow("[+]")
                };
                format!("{}{} {} {}", indent, prefix, icon, tree_line.name)
            } else {
                let size_str = if tree_line.size > 0.0 {
                    format!("  {}", color::dim(&terminal::fmt_size(tree_line.size)))
                } else {
                    String::new()
                };
                format!("{}{} {} {}{}", indent, prefix, color::dim("[f]"), tree_line.name, size_str)
            };

            if i == selection {
                println!("{}", color::cyan(&line));
            } else {
                println!("{}", line);
            }
        }

        let remaining = lines.len().saturating_sub(visible_end);
        if remaining > 0 {
            println!("\n  {} {}", color::dim("v"), color::dim(&format!("{} more", remaining)));
        }

        let hint = if selection < lines.len() && lines[selection].is_directory {
            "Arrows: navigate | Enter: expand/collapse | Esc: back"
        } else if selection < lines.len() {
            "Arrows: navigate | Enter: open/download | Esc: back"
        } else {
            "Arrows: navigate | Esc: back"
        };
        println!("\n  {}", color::dim(hint));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection + 1 < lines.len() {
                    selection += 1;
                }
            }
            "\r" | "\n" => {
                let tree_line = &lines[selection];
                if tree_line.is_directory {
                    if expanded.contains(&tree_line.path) {
                        expanded.remove(&tree_line.path);
                    } else {
                        expanded.insert(tree_line.path.clone());
                    }
                } else {
                    let action = render_modal(&tree_line.name, &tree_line.path, course_id);
                    if action == "open" {
                        if let Some(id) = &tree_line.id {
                            if let Err(e) = open_file(id, course_id, &tree_line.name) {
                                println!("{} {}", color::red("ERR"), e);
                                println!("  {} Press any key.", color::dim("HINT"));
                                terminal::getch();
                            }
                        }
                    } else if action == "download" {
                        if let Some(id) = &tree_line.id {
                            if let Err(e) = download_file_interactive(id, course_id, &tree_line.name) {
                                println!("{} {}", color::red("ERR"), e);
                                println!("  {} Press any key.", color::dim("HINT"));
                                terminal::getch();
                            }
                        }
                    }
                }
            }
            "\x1b" => return Screen::Courses,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}

/* --- Modal ---------------------------------------------------------------- */

fn render_modal(filename: &str, _path: &str, _course_id: u64) -> String {
    let mut selection = 0;
    let options = ["Open", "Download"];

    loop {
        terminal::clear();
        println!("  {}", color::bold("File Action"));
        println!("  {}\n", color::dim("============="));
        println!("  {}", color::dim("File:"));
        println!("  {}\n", color::cyan(filename));

        for (i, opt) in options.iter().enumerate() {
            let prefix = if i == selection { ">" } else { " " };
            if i == selection {
                println!("  {} {}", prefix, color::cyan(opt));
            } else {
                println!("  {} {}", prefix, opt);
            }
        }

        println!("\n  {}", color::dim("Arrows: navigate | Enter: confirm | Esc: cancel"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection < options.len() - 1 {
                    selection += 1;
                }
            }
            "\r" | "\n" => {
                return options[selection].to_lowercase();
            }
            "\x1b" => {
                return "cancel".to_string();
            }
            _ => {}
        }
    }
}

/* --- Open / Download ------------------------------------------------------ */

fn open_file(file_id: &str, course_id: u64, filename: &str) -> Result<(), String> {
    let temp_dir = "/tmp/polito-cli";
    std::fs::create_dir_all(temp_dir).map_err(|e| e.to_string())?;

    let temp_path = format!("{}/{}", temp_dir, filename);

    print!("  Downloading {}...", filename);
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let status = download::download_to_path(course_id, file_id, &temp_path)?;
    println!();
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {}", status));
    }

    util::open_file(&temp_path)
}

fn download_file_interactive(file_id: &str, course_id: u64, filename: &str) -> Result<(), String> {
    let target = if let Some(dir) = util::pick_directory() {
        dir
    } else {
        print!("  {} ", color::dim("Enter target directory:"));
        std::io::Write::flush(&mut std::io::stdout()).map_err(|e| e.to_string())?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;
        input.trim().to_string()
    };

    if target.is_empty() {
        return Err("No target directory specified".to_string());
    }

    let output_path = format!("{}/{}", target, filename);

    print!("  Downloading {}...", filename);
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let status = download::download_to_path(course_id, file_id, &output_path)?;
    println!();
    if !(200..300).contains(&status) {
        return Err(format!("HTTP {}", status));
    }

    println!("{} Downloaded to {}", color::green("v"), output_path);
    Ok(())
}