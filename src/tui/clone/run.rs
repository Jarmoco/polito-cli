/* -----------------------------------------------------------------------------
 * tui/clone_run.rs
 * Clone execution: runs the blocking clone_course with a progress callback that
 * renders a real-time progress bar (block chars), per-file status, and summary.
 * -------------------------------------------------------------------------- */

use std::cell::RefCell;
use std::collections::HashSet;
use std::io::Write;
use std::rc::Rc;

use super::Screen;
use crate::{clone, display::color, terminal};

/* --- Run ------------------------------------------------------------------- */

pub fn run(course_id: u64, course_name: String, shortcode: String, excluded: HashSet<String>) -> Screen {
    let out_dir = format!("polito-{}", course_name.replace(' ', "-").to_lowercase());

    terminal::clear();
    println!("{}", color::bold("  Cloning Course Materials"));
    println!("  {}\n", color::dim("========================="));
    println!("  {} {} ({})", color::dim("Course:"), course_name, shortcode);
    println!("  {} {}", color::dim("Target:"), out_dir);
    println!("\n  {}", color::dim("Preparing..."));
    std::io::stdout().flush().ok();

    let last_progress = Rc::new(RefCell::new(None::<clone::ProgressInfo>));
    let cb_last = last_progress.clone();
    let cb_course = course_name.clone();
    let cb_code = shortcode.clone();
    let cb_dir = out_dir.clone();

    let progress_fn = move |info: &clone::ProgressInfo| {
        *cb_last.borrow_mut() = Some(clone::ProgressInfo {
            files_done: info.files_done,
            files_total: info.files_total,
            bytes_done: info.bytes_done,
            bytes_total: info.bytes_total,
            current_file: info.current_file.clone(),
        });
        render_progress(&cb_course, &cb_code, &cb_dir, info);
    };

    let mut cb: Option<Box<dyn FnMut(&clone::ProgressInfo)>> = Some(Box::new(progress_fn));

    clone::clone_course(clone::CloneOptions {
        course_id,
        course_name: Some(course_name),
        shortcode: Some(shortcode),
        output_dir: Some(out_dir),
        overwrite: false,
        skip_existing: true,
        backup: false,
        progress_cb: cb.take(),
        excluded_paths: Some(excluded),
    });

    let summary = match *last_progress.borrow() {
        Some(ref p) => format!(
            "{} Cloned {} file(s)  ({} / {})",
            color::green("v"),
            p.files_done,
            terminal::fmt_size(p.bytes_done as f64),
            terminal::fmt_size(p.bytes_total as f64),
        ),
        None => format!("{} Cloning complete.", color::green("v")),
    };

    terminal::clear();
    println!("{}", color::bold("  Clone Complete"));
    println!("  {}\n", color::dim("=============="));
    println!("  {}", summary);
    println!("\n  {} Press any key to return to menu.", color::dim("HINT"));
    terminal::getch();
    Screen::Menu
}

/* --- Progress Rendering --------------------------------------------------- */

fn render_progress(course_name: &str, shortcode: &str, out_dir: &str, info: &clone::ProgressInfo) {
    terminal::clear();

    println!("{}", color::bold("  Cloning Course Materials"));
    println!("  {}\n", color::dim("========================="));
    println!("  {} {} ({})", color::dim("Course:"), course_name, shortcode);
    println!("  {} {}\n", color::dim("Target:"), out_dir);

    let pct = if info.bytes_total > 0 {
        let raw = info.bytes_done as f64 / info.bytes_total as f64 * 100.0;
        (raw as usize).min(100)
    } else if info.files_total > 0 {
        info.files_done * 100 / info.files_total
    } else {
        0
    };

    println!("  {}\n", progress_bar(pct, 40));

    println!(
        "  {}  File {}/{}  |  {} / {}",
        color::dim("Files:"),
        info.files_done,
        info.files_total,
        terminal::fmt_size(info.bytes_done as f64),
        terminal::fmt_size(info.bytes_total as f64),
    );

    println!("  {}  {}", color::dim("Current:"), info.current_file);
    std::io::stdout().flush().ok();
}

/* --- Progress Bar --------------------------------------------------------- */

fn progress_bar(pct: usize, width: usize) -> String {
    let filled = (pct * width).min(width * 100) / 100;
    let empty = width.saturating_sub(filled);
    let mut bar = String::new();
    bar.push('[');
    for _ in 0..filled {
        bar.push('█');
    }
    for _ in 0..empty {
        bar.push('▒');
    }
    bar.push(']');
    format!("{}  {}%", bar, pct)
}
