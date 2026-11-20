use crate::style::{PAD, TAG_DEBUG, TAG_ERROR, TAG_INFO, TAG_OK, TAG_WARN};
use colored::Colorize;
use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Enable debug output (`--verbose` or `SIT_DEBUG=1`).
pub fn set_verbose(on: bool) {
    VERBOSE.store(on, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Debug trace to stderr, only when verbose:  `  debug: msg`
pub fn debug(msg: &str) {
    if is_verbose() {
        eprintln!("{PAD}{} {}", TAG_DEBUG.dimmed(), msg.dimmed());
    }
}

/// Print a success message:  `  ok: msg`
pub fn success(msg: &str) {
    println!("{PAD}{} {}", TAG_OK.green(), msg.bold());
}

/// Print a success message with detail lines underneath.
pub fn success_with_details(msg: &str, details: &str) {
    success(msg);
    for line in details.lines() {
        detail(line);
    }
}

/// Print an error message to stderr:  `  error: msg`
pub fn error(msg: &str) {
    eprintln!("{PAD}{} {}", TAG_ERROR.red(), msg);
}

/// Print a warning message:  `  warn: msg`
pub fn warn(msg: &str) {
    eprintln!("{PAD}{} {}", TAG_WARN.yellow(), msg);
}

/// Print an informational message:  `  info: msg`
pub fn info(msg: &str) {
    println!("{PAD}{} {}", TAG_INFO.blue(), msg);
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
