/* -----------------------------------------------------------------------------
 * util/fs.rs
 * File operations: open with xdg-open/open, directory picker via zenity/kdialog.
 * -------------------------------------------------------------------------- */

use std::process::Command;

/* --- Open File ------------------------------------------------------------ */

pub fn open_file(path: &str) -> Result<(), String> {
    let result = Command::new("xdg-open")
        .arg(path)
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            return Ok(());
        }
    }

    let result = Command::new("open")
        .arg(path)
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            return Ok(());
        }
    }

    Err("No file opener found".to_string())
}

/* --- Pick Directory -------------------------------------------------------- */

pub fn pick_directory() -> Option<String> {
    let result = Command::new("zenity")
        .args(&["--file-selection", "--directory"])
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

    let result = Command::new("kdialog")
        .args(&["--getexistingdirectory"])
        .output();

    if let Ok(output) = result {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                return Some(path);
            }
        }
    }

    None
}
