use crate::style::{PAD, TAG_ERROR, TAG_INFO, TAG_OK, TAG_WARN};
use colored::Colorize;

/// Print a success message:  `  [OK] msg`
pub fn success(msg: &str) {
    println!("{PAD}{} {}", TAG_OK.green().bold(), msg.bold());
}

/// Print a success message with detail lines underneath.
pub fn success_with_details(msg: &str, details: &str) {
    success(msg);
    for line in details.lines() {
        detail(line);
    }
}

/// Print an error message to stderr:  `  [ERROR] msg`
pub fn error(msg: &str) {
    eprintln!("{PAD}{} {}", TAG_ERROR.red().bold(), msg);
}

/// Print a warning message:  `  [WARN] msg`
pub fn warn(msg: &str) {
    eprintln!("{PAD}{} {}", TAG_WARN.yellow().bold(), msg.yellow());
}

/// Print an informational message:  `  [INFO] msg`
pub fn info(msg: &str) {
    println!("{PAD}{} {}", TAG_INFO.blue().bold(), msg);
}

/// Print a dimmed hint line:  `  msg`
pub fn hint(msg: &str) {
    println!("{PAD}{}", msg.dimmed());
}

/// Print an indented, dimmed detail line:  `    msg`
pub fn detail(msg: &str) {
    println!("{PAD}  {}", msg.dimmed());
}

/// Print a bold header:  `  msg`
pub fn header(msg: &str) {
    println!("{PAD}{}", msg.bold());
}

/// Print an empty line for visual spacing.
pub fn blank() {
    println!();
}
