/* -----------------------------------------------------------------------------
 * config.rs
 * Manages token and username storage via XDG ~/.config/polito-cli/ directory
 * with POLITO_TOKEN env var as primary source.
 * -------------------------------------------------------------------------- */

use std::env;
use std::fs;
use std::path::PathBuf;

use crate::error::PoliError;

/* --- Constants ------------------------------------------------------------ */

const TOKEN_FILE: &str = ".token";
const USERNAME_FILE: &str = ".user";

/* --- Paths ---------------------------------------------------------------- */

fn config_dir() -> PathBuf {
    env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            env::var("HOME")
                .map(|h| PathBuf::from(h).join(".config"))
                .unwrap_or_default()
        })
        .join("polito-cli")
}

fn token_path() -> PathBuf {
    config_dir().join(TOKEN_FILE)
}

fn username_path() -> PathBuf {
    config_dir().join(USERNAME_FILE)
}

/* --- Token ----------------------------------------------------------------- */

pub fn save_token(token: &str) -> Result<(), PoliError> {
    let dir = config_dir();
    fs::create_dir_all(&dir).map_err(|e| PoliError::Config(e.to_string()))?;
    fs::write(token_path(), token).map_err(|e| PoliError::Config(e.to_string()))?;
    env::set_var("POLITO_TOKEN", token);
    Ok(())
}

pub fn load_token() -> Result<Option<String>, PoliError> {
    if let Ok(token) = env::var("POLITO_TOKEN") {
        if !token.is_empty() {
            return Ok(Some(token));
        }
    }
    let p = token_path();
    if p.exists() {
        Ok(Some(
            fs::read_to_string(&p).map_err(|e| PoliError::Config(e.to_string()))?,
        ))
    } else {
        Ok(None)
    }
}

pub fn delete_token() -> Result<(), PoliError> {
    let path = token_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| PoliError::Config(e.to_string()))?;
    }
    env::remove_var("POLITO_TOKEN");
    Ok(())
}

/* --- Username ------------------------------------------------------------- */

pub fn save_username(username: &str) -> Result<(), PoliError> {
    fs::write(username_path(), username).map_err(|e| PoliError::Config(e.to_string()))
}

pub fn delete_username() -> Result<(), PoliError> {
    let path = username_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| PoliError::Config(e.to_string()))?;
    }
    Ok(())
}

/* --- Mock URL -------------------------------------------------------------- */

pub fn get_mock_url() -> Option<String> {
    env::var("POLITO_MOCK_URL").ok().filter(|s| !s.is_empty())
}
