/* -----------------------------------------------------------------------------
 * tui/menu.rs
 * Main menu screen: arrow navigation, Enter selects, Esc/q quits.
 * -------------------------------------------------------------------------- */

use super::Screen;
use crate::display::color;
use crate::terminal;

/* --- Items ---------------------------------------------------------------- */

const ITEMS: &[(&str, Screen)] = &[
    ("Courses", Screen::Courses),
    ("Exams", Screen::Exams),
    ("Grades", Screen::Grades),
    ("Lectures", Screen::Lectures),
    ("Clone", Screen::Clone),
    ("Quit", Screen::Quit),
];

/* --- Render --------------------------------------------------------------- */

pub fn render() -> Screen {
    let mut selection = 0;

    loop {
        terminal::clear();
        println!("{}", color::bold("  Polito CLI"));
        println!("  {}\n", color::dim("============"));

        for (i, (label, _)) in ITEMS.iter().enumerate() {
            if i == selection {
                println!("  > {}", color::cyan(label));
            } else {
                println!("    {}", label);
            }
        }

        println!("\n  {}", color::dim("Arrows: navigate | Enter: select | q/Esc: quit"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if selection > 0 {
                    selection -= 1;
                }
            }
            "\x1b[B" => {
                if selection < ITEMS.len() - 1 {
                    selection += 1;
                }
            }
            "\r" | "\n" => {
                let (_, s) = ITEMS[selection];
                return s;
            }
            "q" | "\x1b" => return Screen::Quit,
            _ => {}
        }
    }
}
