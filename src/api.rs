/* -----------------------------------------------------------------------------
 * api.rs
 * HTTP wrappers around curl via std::process::Command. Provides get, post,
 * delete, download_file operations with configurable BASE_URL.
 * -------------------------------------------------------------------------- */

use std::process::Command;

use crate::{config, error::PoliError};

/* --- Base URL ------------------------------------------------------------- */

const PROD_URL: &str = "https://app.didattica.polito.it/api";

fn get_base_url() -> String {
    if let Some(mock_url) = config::get_mock_url() {
        mock_url
    } else {
        PROD_URL.to_string()
    }
}

pub fn get_api_base_url() -> Result<String, PoliError> {
    let _ = config::load_token()?.ok_or(PoliError::NotLoggedIn)?;
    Ok(get_base_url())
}

/* --- Types ---------------------------------------------------------------- */

pub struct ApiResponse {
    pub status: u16,
    pub body: String,
}

/* --- Authenticated Requests ----------------------------------------------- */

pub fn get(path: &str) -> Result<ApiResponse, PoliError> {
    let token = config::load_token()?.ok_or(PoliError::NotLoggedIn)?;
    let url = format!("{}{}", get_base_url(), path);
    curl_get(&url, &token)
}

pub fn delete(path: &str) -> Result<ApiResponse, PoliError> {
    let token = config::load_token()?.ok_or(PoliError::NotLoggedIn)?;
    let url = format!("{}{}", get_base_url(), path);
    curl_delete(&url, &token)
}

/* --- Unauthenticated Requests --------------------------------------------- */

pub fn post_unauthenticated(path: &str, body: &str) -> Result<ApiResponse, PoliError> {
    curl_post_raw(&format!("{}{}", get_base_url(), path), body)
}

/* --- Curl Helpers --------------------------------------------------------- */

fn curl_get(url: &str, token: &str) -> Result<ApiResponse, PoliError> {
    let output = Command::new("curl")
        .args(&[
            "-X",
            "GET",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "Accept: application/json",
            "-s",
            "-w",
            "%{http_code}",
            url,
        ])
        .output()
        .map_err(|e| PoliError::Network(e.to_string()))?;
    parse_response(output)
}

fn curl_post_raw(url: &str, body: &str) -> Result<ApiResponse, PoliError> {
    let output = Command::new("curl")
        .args(&[
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-H",
            "Accept: application/json",
            "-d",
            body,
            "-s",
            "-w",
            "%{http_code}",
            url,
        ])
        .output()
        .map_err(|e| PoliError::Network(e.to_string()))?;
    parse_response(output)
}

fn curl_delete(url: &str, token: &str) -> Result<ApiResponse, PoliError> {
    let output = Command::new("curl")
        .args(&[
            "-X",
            "DELETE",
            "-H",
            &format!("Authorization: Bearer {}", token),
            "-H",
            "Content-Type: application/json",
            "-s",
            "-w",
            "%{http_code}",
            url,
        ])
        .output()
        .map_err(|e| PoliError::Network(e.to_string()))?;
    parse_response(output)
}

/* --- Response Parsing ----------------------------------------------------- */

fn parse_response(output: std::process::Output) -> Result<ApiResponse, PoliError> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let output_str = stdout.trim();
    // curl -w "%{http_code}" appends the 3-digit status code at the END
    let (body, status_str) = if output_str.len() >= 3 {
        output_str.split_at(output_str.len() - 3)
    } else {
        ("", output_str)
    };
    let status: u16 = status_str.parse().unwrap_or(500);
    if !output.status.success() {
        return Err(PoliError::Network(stderr.to_string()));
    }
    Ok(ApiResponse { status, body: body.to_string() })
}
