/* -----------------------------------------------------------------------------
 * tui/mod.rs
 * Main TUI event loop: alternate screen, render/dispatch per Screen variant.
 * -------------------------------------------------------------------------- */

use crate::terminal;

mod clone;
mod courses;
mod exams;
mod files;
mod grades;
mod lectures;
mod menu;
mod notices;

/* --- Screen --------------------------------------------------------------- */

#[derive(Clone, Copy, PartialEq)]
pub enum Screen {
    Menu,
    Courses,
    Files(u64),
    Notices(u64),
    Exams,
    Lectures,
    Grades,
    Clone,
    Quit,
}

/* --- Run ------------------------------------------------------------------ */

pub fn run() {
    terminal::init_signal_handler();
    terminal::enter_alternate_screen();
    let mut screen = Screen::Menu;

    loop {
        terminal::clear();
        match screen {
            Screen::Menu => screen = menu::render(),
            Screen::Courses => screen = courses::render(),
            Screen::Files(course_id) => screen = files::render(course_id),
            Screen::Notices(course_id) => screen = notices::render(course_id),
            Screen::Exams => screen = exams::render(),
            Screen::Lectures => screen = lectures::render(),
            Screen::Grades => screen = grades::render(),
            Screen::Clone => screen = clone::render(),
            Screen::Quit => break,
        }
    }

    terminal::exit_alternate_screen();
}
