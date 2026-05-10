/* -----------------------------------------------------------------------------
 * download.rs
 * File download functions: single file download and recursive directory
 * download. Uses flatten_files from util to traverse the file tree.
 * -------------------------------------------------------------------------- */

use std::io::Write;
use std::process::Command;

use crate::{api, config, data, display::color, error::PoliError, util};

/* --- Single File ---------------------------------------------------------- */

pub fn download_file(
    course_id: u64,
    file_id: &str,
    output_dir: Option<&str>,
) -> Result<(), PoliError> {
    let out_dir = output_dir.unwrap_or(".");
    std::fs::create_dir_all(out_dir)
        .map_err(|e| PoliError::Config(e.to_string()))?;
    let base_url = api::get_api_base_url()?;
    let url = format!("{}/courses/{}/files/{}", base_url, course_id, file_id);
    let token = config::load_token()?.ok_or(PoliError::NotLoggedIn)?;
    let output_path = format!("{}/{}", out_dir, file_id);

    println!("  Downloading {}...", file_id);

    let result = Command::new("curl")
        .args(&[
            "-L",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-o",
            &output_path,
            "-#",
            "-w",
            "%{http_code}",
            &url,
        ])
        .output()
        .map_err(|e| PoliError::Network(e.to_string()))?;
    let status: u16 = String::from_utf8_lossy(&result.stdout)
        .chars()
        .take(3)
        .collect::<String>()
        .parse()
        .unwrap_or(200);
    if status == 404 {
        return Err(PoliError::FileNotFound(file_id.to_string()));
    }
    if status >= 400 {
        return Err(PoliError::Api {
            status,
            message: status.to_string(),
        });
    }
    println!("\r{} Downloaded to {}", color::green("v"), output_path);
    Ok(())
}

/* --- All Files ------------------------------------------------------------ */

pub fn download_all(course_id: u64, output_dir: Option<&str>) -> Result<(), PoliError> {
    let default_dir = format!("polito-{}", course_id);
    let out_dir = output_dir.unwrap_or(&default_dir);
    std::fs::create_dir_all(out_dir).map_err(|e| PoliError::Config(e.to_string()))?;

    let entries = data::fetch_files(course_id)
        .map_err(|e| PoliError::Api { status: 0, message: e })?;
    let flat = util::flatten_files(&entries, "");

    if flat.is_empty() {
        println!("{} No files found.", color::yellow("!"));
        return Ok(());
    }

    println!(
        "Downloading {} file(s) into {}...",
        flat.len(),
        out_dir
    );

    let base_url = api::get_api_base_url()?;
    let token = config::load_token()?.ok_or(PoliError::NotLoggedIn)?;

    for (rel_path, file) in &flat {
        let dest = format!("{}/{}", out_dir, rel_path);
        if std::path::Path::new(&dest).exists() {
            println!("  {} (skipped)", color::dim(rel_path));
            continue;
        }
        let file_id = file["id"].as_str().unwrap_or("");
        let url = format!("{}/courses/{}/files/{}", base_url, course_id, file_id);
        if let Some(p) = std::path::Path::new(&dest).parent() {
            if let Err(e) = std::fs::create_dir_all(p) {
                eprintln!("{} Failed to create directory {}: {}", color::red("x"), p.display(), e);
                return Err(PoliError::Config(e.to_string()));
            }
        }

        print!("  {} ", color::dim(rel_path));
        let _ = std::io::stdout().flush(); // best-effort: status display

        let result = Command::new("curl")
            .args(&[
                "-L",
                "-H",
                &format!("Authorization: Bearer {}", token),
                "-o",
                &dest,
                "-#",
                "-w",
                "%{http_code}",
                &url,
            ])
            .output();

        match result {
            Ok(output) => {
                let status: u16 = String::from_utf8_lossy(&output.stdout)
                    .chars()
                    .take(3)
                    .collect::<String>()
                    .parse()
                    .unwrap_or(200);
                if status >= 200 && status < 400 {
                    println!("\r  {} {}", color::green("v"), rel_path);
                } else {
                    println!("\r  {} {} (HTTP {})", color::red("x"), rel_path, status);
                }
            }
            Err(e) => {
                println!("\r  {} {} ({})", color::red("x"), rel_path, e);
            }
        }
    }

    println!("\n{} Done - files saved to {}", color::green("v"), out_dir);
    Ok(())
}

/* --- Shared Download Primitive --------------------------------------------- */

pub fn download_to_path(course_id: u64, file_id: &str, dest: &str) -> Result<u16, String> {
    let base_url = api::get_api_base_url().map_err(|e| e.to_string())?;
    let token = config::load_token()
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Not logged in".to_string())?;
    let url = format!("{}/courses/{}/files/{}", base_url, course_id, file_id);
    let output = Command::new("curl")
        .args(&[
            "-L",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-o",
            dest,
            "-s",
            "-w",
            "%{http_code}",
            &url,
        ])
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;
    let status: u16 = String::from_utf8_lossy(&output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    // API may return HTTP 200 with an HTML error page
    if (200..300).contains(&status) {
        if let Ok(content) = std::fs::read_to_string(dest) {
            if content.contains("<html") || content.contains("Error ") {
                let _ = std::fs::remove_file(dest);
                return Err("Server returned HTML error page".to_string());
            }
        }
    }
    Ok(status)
}
