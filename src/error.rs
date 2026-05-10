/* -----------------------------------------------------------------------------
 * error.rs
 * Defines the PoliError enum for all domain-specific error conditions.
 * -------------------------------------------------------------------------- */

use std::fmt;

use crate::json::ParseError;

/* --- Error Enum ----------------------------------------------------------- */

#[derive(Debug)]
pub enum PoliError {
    NotLoggedIn,
    Api { status: u16, message: String },
    Network(String),
    Config(String),
    FileNotFound(String),
    Parse(String),
}

/* --- Display -------------------------------------------------------------- */

impl fmt::Display for PoliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            PoliError::NotLoggedIn => write!(f, "Not logged in. Run `polito login` first."),
            PoliError::Api { status, message } => write!(f, "API error {}: {}", status, message),
            PoliError::Network(msg) => write!(f, "Network error: {}", msg),
            PoliError::Config(msg) => write!(f, "Config error: {}", msg),
            PoliError::FileNotFound(id) => write!(f, "File not found: {}", id),
            PoliError::Parse(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

impl std::error::Error for PoliError {}

impl From<ParseError> for PoliError {
    fn from(e: ParseError) -> Self {
        PoliError::Parse(e.to_string())
    }
}
