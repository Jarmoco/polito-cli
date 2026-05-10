/* -----------------------------------------------------------------------------
 * display/color.rs
 * Single source of truth for ANSI escape color helpers. Every module
 * imports from here — never duplicate color definitions.
 * -------------------------------------------------------------------------- */

pub fn green(s: &str) -> String { format!("\x1b[32m{}\x1b[0m", s) }
pub fn red(s: &str) -> String { format!("\x1b[31m{}\x1b[0m", s) }
pub fn yellow(s: &str) -> String { format!("\x1b[33m{}\x1b[0m", s) }
pub fn cyan(s: &str) -> String { format!("\x1b[36m{}\x1b[0m", s) }
pub fn dim(s: &str) -> String { format!("\x1b[90m{}\x1b[0m", s) }
pub fn bold(s: &str) -> String { format!("\x1b[1m{}\x1b[0m", s) }
