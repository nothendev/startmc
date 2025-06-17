use std::fmt::Display;

use owo_colors::OwoColorize;

pub fn cols() -> String {
    "::".blue().bold().to_string()
}

pub fn arrow_error<'a>(text: impl Display) {
    println!(
        "{} {}",
        "==> ERROR:".red().bold(),
        text.bold()
    )
}

pub fn arrow_warn<'a>(text: impl Display) {
    println!(
        "{} {}",
        "==> WARNING:".yellow().bold(),
        text.bold()
    )
}

pub fn pacman_warn(text: impl Display) {
    println!("{} {text}", "warning:".yellow().bold())
}

pub fn green_arrow() -> String {
    "==>".green().bold().to_string()
}
