/* -----------------------------------------------------------------------------
 * cli.rs
 * Manual CLI argument parser and command dispatcher (no clap crate).
 * -------------------------------------------------------------------------- */

use std::env;
use std::io::{self, Write};

use crate::{auth, clone, display, download, terminal, tui};

/* --- Commands ------------------------------------------------------------- */

pub enum Commands {
    Login,
    Logout,
    Whoami,
    Courses { year: Option<String>, json: bool },
    Files { course_id: u64, list: bool, path: Option<String>, output: Option<String>, all: bool, json: bool, size: bool },
    Notices { course_id: u64, json: bool },
    Exams { json: bool },
    Lectures { from: Option<String>, to: Option<String>, course_id: Option<u64>, json: bool },
    Grades { json: bool },
    Clone { course_id: u64, output: Option<String>, overwrite: bool, skip_existing: bool, backup: bool },
    Mock,
}

/* --- Dispatch ------------------------------------------------------------- */

pub fn dispatch() {
    let args: Vec<String> = env::args().collect();
    let command = parse(&args);
    match command {
        Commands::Login => auth::login(),
        Commands::Logout => auth::logout(),
        Commands::Whoami => auth::whoami(),
        Commands::Courses { year, json } => display::courses::show_courses(year.as_deref(), json),
        Commands::Files { course_id, list, path, output, all, json, size } => {
            if size {
                match clone::get_course_size(course_id) {
                    Ok((count, bytes)) => {
                        if json {
                            println!("{{\"fileCount\":{},\"totalBytes\":{}}}", count, bytes);
                        } else {
                            println!("{} file(s), {}", count, terminal::fmt_size(bytes as f64));
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            } else if all {
                if let Err(e) = download::download_all(course_id, output.as_deref()) {
                    eprintln!("Download failed: {}", e);
                }
            } else if list {
                display::files::show_files(course_id, path.as_deref(), json);
            } else if let Some(ref file_id_or_name) = path {
                if download::download_file(course_id, file_id_or_name, output.as_deref()).is_err() {
                    display::files::show_files(course_id, None, false);
                    eprintln!("Hint: try `polito files {} --path <FILE_ID>`", course_id);
                }
            } else {
                display::files::show_files(course_id, None, json);
            }
        }
        Commands::Notices { course_id, json } => {
            display::notices::show_notices(course_id, json);
        }
        Commands::Exams { json } => display::exams::show_exams(json),
        Commands::Lectures { from, to, course_id, json } => {
            display::lectures::show_lectures(from.as_deref(), to.as_deref(), course_id, json);
        }
        Commands::Grades { json } => display::grades::show_grades(json),
        Commands::Clone { course_id, output, overwrite, skip_existing, backup } => {
            clone::clone_course(clone::CloneOptions {
                course_id,
                course_name: None,
                shortcode: None,
                output_dir: output,
                overwrite,
                skip_existing,
                backup,
                progress_cb: None,
                excluded_paths: None,
            });
        }
        Commands::Mock => run_mock(),
    }
}

/* --- Parser --------------------------------------------------------------- */

fn parse(args: &[String]) -> Commands {
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    if args[1] == "-h" || args[1] == "--help" {
        print_help();
        std::process::exit(0);
    }
    if args[1] == "-V" || args[1] == "--version" {
        println!("polito {}", env!("CARGO_PKG_VERSION"));
        std::process::exit(0);
    }

    let json = has_flag(args, "--json");
    match args[1].as_str() {
        "login" => Commands::Login,
        "logout" => Commands::Logout,
        "whoami" => Commands::Whoami,
        "courses" => Commands::Courses { year: get_flag(args, "--year"), json },
        "files" => Commands::Files {
            course_id: get_required(args, 2),
            list: has_flag(args, "--list"),
            path: get_flag(args, "--path"),
            output: get_flag(args, "--output"),
            all: has_flag(args, "--all"),
            size: has_flag(args, "--size"),
            json,
        },
        "notices" => Commands::Notices { course_id: get_required(args, 2), json },
        "exams" => Commands::Exams { json },
        "lectures" => Commands::Lectures {
            from: get_flag(args, "--from"),
            to: get_flag(args, "--to"),
            course_id: get_flag(args, "--course-id"),
            json,
        },
        "grades" => Commands::Grades { json },
        "clone" => Commands::Clone {
            course_id: get_required(args, 2),
            output: get_flag(args, "--output"),
            overwrite: has_flag(args, "--overwrite"),
            skip_existing: has_flag(args, "--skip-existing"),
            backup: has_flag(args, "--backup"),
        },
        "mock" => Commands::Mock,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

/* --- Helpers -------------------------------------------------------------- */

fn print_usage() {
    eprintln!("Usage: polito <command> [args]");
    eprintln!("Run 'polito --help' for more details.");
}

fn print_help() {
    println!("Usage: polito <command> [options]");
    println!();
    println!("Commands:");
    println!("  login              Log in with PoliTo credentials");
    println!("  logout             Log out and clear stored credentials");
    println!("  whoami             Show current user info");
    println!("  courses [opts]     List enrolled courses");
    println!("  files <ID> [opts]  List/download course files");
    println!("  notices <ID>       List course notices");
    println!("  exams              List upcoming exams");
    println!("  lectures [opts]    List lecture schedule");
    println!("  grades             Show recorded grades");
    println!("  clone <ID> [opts]  Clone course files locally");
    println!("  mock               Launch TUI with mock API");
    println!();
    println!("Options:");
    println!("  --json             Output as JSON");
    println!("  --year <YEAR>      Filter by academic year");
    println!("  --path <FILE>      Specify file by name or ID");
    println!("  --output <DIR>     Output directory");
    println!("  --all              Download all files");
    println!("  --size             Show total file count and size");
    println!("  --overwrite        Overwrite existing files");
    println!("  --skip-existing   Skip existing files");
    println!("  --backup           Backup existing files");
    println!("  --from <DATE>      Filter lectures from date");
    println!("  --to <DATE>        Filter lectures to date");
    println!();
    println!("Environment variables:");
    println!("  POLITO_MOCK_URL    Set mock API URL for testing");
    println!();
    println!("Run 'polito' without args to launch the TUI.");
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.contains(&flag.to_string())
}

fn get_flag<T: std::str::FromStr>(args: &[String], flag: &str) -> Option<T> {
    for (i, arg) in args.iter().enumerate() {
        if arg == flag && i + 1 < args.len() {
            return args[i + 1].parse().ok();
        }
    }
    None
}

fn get_required<T: std::str::FromStr>(args: &[String], index: usize) -> T {
    if args.len() > index {
        args[index].parse().unwrap_or_else(|_| {
            eprintln!("Missing required argument at position {}", index);
            std::process::exit(1);
        })
    } else {
        eprintln!("Missing required argument");
        std::process::exit(1);
    }
}

/* --- Mock Mode ------------------------------------------------------------- */

fn run_mock() {
    print!("Mock API URL: ");
    if io::stdout().flush().is_err() {
        eprintln!("Failed to flush stdout");
        std::process::exit(1);
    }
    let mut url = String::new();
    if io::stdin().read_line(&mut url).is_err() {
        eprintln!("Failed to read URL");
        std::process::exit(1);
    }
    let url = url.trim();
    if url.is_empty() {
        eprintln!("Mock URL cannot be empty");
        std::process::exit(1);
    }
    env::set_var("POLITO_MOCK_URL", url);
    tui::run();
}
