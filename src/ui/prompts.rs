use inquire::Text;

use super::editor::{EditorOptions, edit};
use crate::error::Result;
use crate::style::{GREY, RESET};

fn badge(label: &str, fg: u8) -> (String, usize) {
    (
        format!("\x1b[1;7;{fg}m {label} {RESET} "),
        label.chars().count() + 3,
    )
}

fn non_empty_message(raw: String) -> Result<String> {
    let message = raw.trim().to_string();
    if message.is_empty() {
        return Err("Commit message cannot be empty".into());
    }
    Ok(message)
}

pub fn prompt_message(category: &str) -> Result<String> {
    let (prefix, prefix_width) = badge(category, 36);
    non_empty_message(edit(&EditorOptions {
        prefix: &prefix,
        prefix_width,
        initial: "",
        placeholder: "enter commit message...",
        multiline: false,
        esc_skips: false,
    })?)
}

/// Optional multi-line description. Enter submits; Esc or empty input skips.
pub fn prompt_description() -> Result<String> {
    let prefix = format!("{GREY}    {RESET}Description (optional): ");
    let text = edit(&EditorOptions {
        prefix: &prefix,
        prefix_width: 4 + "Description (optional): ".len(),
        initial: "",
        placeholder: "",
        multiline: true,
        esc_skips: true,
    })?;
    Ok(text.trim_end().to_string())
}

/// Prompt for a commit message pre-filled with the last commit's message
/// (subject and body). Edit it, or press Enter to keep it unchanged.
pub fn prompt_amend_message(current: &str) -> Result<String> {
    let (prefix, prefix_width) = badge("amend", 33);
    non_empty_message(edit(&EditorOptions {
        prefix: &prefix,
        prefix_width,
        initial: current,
        placeholder: "",
        multiline: true,
        esc_skips: false,
    })?)
}

pub fn prompt_release_tag(previous_tag: Option<&str>) -> Result<String> {
    let mut text = Text::new("Release version:");

    let previous_clean =
        previous_tag.map(|previous| previous.trim().trim_start_matches('v').to_string());
    if let Some(previous) = previous_clean.as_deref()
        && !previous.is_empty()
    {
        text = text.with_placeholder(previous);
        // text = text.with_help_message("Use semantic version format: 1.2.3");
    }

    let value = text.prompt()?;
    let value = value.trim();

    if value.is_empty() {
        return Err("Release version cannot be empty".into());
    }

    let normalized = if value.starts_with('v') {
        value.to_string()
    } else {
        format!("v{value}")
    };

    Ok(normalized)
}

pub fn print_success(commit_msg: &str) {
    crate::print::blank();
    crate::print::success_with_details("Committed", commit_msg);
    crate::print::blank();
}

pub fn print_error(msg: &str) {
    crate::print::blank();
    crate::print::error(msg);
    crate::print::blank();
}
