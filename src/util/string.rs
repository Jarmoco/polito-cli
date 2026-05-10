/* -----------------------------------------------------------------------------
 * util/string.rs
 * String truncation helper.
 * -------------------------------------------------------------------------- */

/* --- Truncate ------------------------------------------------------------- */

pub fn trunc(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}…", s.chars().take(max - 1).collect::<String>())
    } else {
        s.to_string()
    }
}
