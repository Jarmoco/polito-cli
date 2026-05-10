/* -----------------------------------------------------------------------------
 * util/html.rs
 * Strips HTML tags and ANSI escape codes from strings.
 * -------------------------------------------------------------------------- */

/* --- Strip HTML ----------------------------------------------------------- */

pub fn strip_html(s: &str) -> String {
    let mut result = s.to_string();
    loop {
        let start = match result.find('<') {
            Some(idx) => idx,
            None => break,
        };
        if let Some(end) = result[start..].find('>') {
            result.replace_range(start..start + end + 1, "");
        } else {
            break;
        }
    }
    strip_ansi(&result)
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if chars.as_str().starts_with('[') {
                chars.next();
                while let Some(n) = chars.next() {
                    if n >= '\x40' && n <= '\x7e' {
                        break;
                    }
                }
            } else {
                result.push(c);
            }
        } else {
            result.push(c);
        }
    }
    result.trim().to_string()
}
