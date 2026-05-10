/* -----------------------------------------------------------------------------
 * auth.rs
 * login, logout, and whoami commands. Login reads stdin prompts for
 * credentials, logout performs server logout + clears local state.
 * -------------------------------------------------------------------------- */

use std::io::{self, Write};

use crate::{api, config, display::color, json};

/* --- Login ---------------------------------------------------------------- */

pub fn login() {
    let username = prompt("Username (s-number): ");
    let password = prompt_password("Password: ");
    let body = format!(
        r#"{{"username":"{}","password":"{}","loginType":"basic","preferences":{{}},"device":{{"platform":"linux","version":"1.0.0","model":"cli","toothPicCompatible":false}}}}"#,
        username, password
    );
    match api::post_unauthenticated("/auth/login", &body) {
        Ok(resp) if resp.status == 200 => {
            let json = match json::parse(&resp.body) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    return;
                }
            };
            let token = json["data"]["token"].as_str().unwrap_or("");
            let username_val = json["data"]["username"].as_str().unwrap_or("");
            if config::save_token(token).is_err() || config::save_username(username_val).is_err() {
                eprintln!("Failed to save credentials");
                return;
            }
            let name = json["data"]["name"].as_str();
            let surname = json["data"]["surname"].as_str();
            let display_name = match (name, surname) {
                (Some(n), Some(s)) => format!("{} {}", n, s),
                (Some(n), None) => n.to_string(),
                _ => username_val.to_string(),
            };
            println!(
                "{} Logged in as {} ({})",
                color::green("OK"),
                color::bold(&display_name),
                color::dim(username_val)
            );
        }
        Ok(resp) => {
            let body_preview = if resp.body.len() > 500 {
                format!("{}... (truncated)", &resp.body[..500])
            } else {
                resp.body.clone()
            };
            // Try to extract message from JSON error body; fall back to raw body
            let error_message = json::parse(&resp.body)
                .ok()
                .and_then(|v| v["message"].as_str().map(String::from))
                .unwrap_or_else(|| format!("HTTP {} — body: {}", resp.status, body_preview));
            eprintln!("{} Authentication failed: {}", color::red("ERR"), error_message);
        }
        Err(e) => {
            eprintln!("{} Network error: {}", color::red("ERR"), e);
        }
    }
}

/* --- Logout --------------------------------------------------------------- */

pub fn logout() {
    if let Err(e) = api::delete("/auth/logout") {
        eprintln!("{} Logout request failed: {}", color::dim("warn"), e);
    }
    if config::delete_token().is_err() {
        eprintln!("{} Failed to clear token.", color::dim("warn"));
    }
    if config::delete_username().is_err() {
        eprintln!("{} Failed to clear username.", color::dim("warn"));
    }
    println!("{} Logged out.", color::green("OK"));
}

/* --- Whoami --------------------------------------------------------------- */

pub fn whoami() {
    match api::get("/me") {
        Ok(resp) if resp.status == 200 => {
            let json = match json::parse(&resp.body) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Parse error: {}", e);
                    return;
                }
            };
            let data = &json["data"];
            println!();
            println!(
                "  {}  {} {}",
                color::dim("Name:"),
                data["firstName"].as_str().unwrap_or("-"),
                data["lastName"].as_str().unwrap_or("-")
            );
            println!(
                "  {} {}",
                color::dim("Username:"),
                data["username"].as_str().unwrap_or("-")
            );
            println!(
                "  {}    {}",
                color::dim("Email:"),
                data["email"].as_str().unwrap_or("-")
            );
            println!(
                "  {}       {}",
                color::dim("ID:"),
                data["studentId"].as_str().unwrap_or("-")
            );
            println!();
        }
        Ok(resp) => {
            let body_preview = if resp.body.len() > 200 {
                format!("{}... (truncated)", &resp.body[..200])
            } else {
                resp.body.clone()
            };
            eprintln!("HTTP error {} — {}", resp.status, body_preview);
        }
        Err(e) => eprintln!("{}", e),
    }
}

/* --- Prompts -------------------------------------------------------------- */

fn prompt(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush(); // best-effort: prompt is non-critical
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return String::new();
    }
    input.trim().to_string()
}

fn prompt_password(msg: &str) -> String {
    print!("{}", msg);
    let _ = io::stdout().flush(); // best-effort: prompt is non-critical
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return String::new();
    }
    input.trim().to_string()
}
