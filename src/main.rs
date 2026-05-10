/* -----------------------------------------------------------------------------
 * main.rs
 * Entry point: no args → TUI mode, args → CLI mode.
 * -------------------------------------------------------------------------- */

mod api;
mod auth;
mod cli;
mod clone;
mod config;
mod conflict;
mod data;
mod display;
mod download;
mod error;
mod json;
mod meta;
mod terminal;
mod tui;
mod util;

fn main() {
    std::panic::set_hook(Box::new(|_| {
        if terminal::ALTERNATE_MODE.load(std::sync::atomic::Ordering::SeqCst) {
            terminal::exit_alternate_screen();
        }
    }));

    terminal::init_signal_handler();
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 1 {
        tui::run();
    } else {
        cli::dispatch();
    }
}
