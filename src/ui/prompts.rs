use inquire::Text;
use inquire::ui::{Color, RenderConfig, Styled};

pub fn prompt_message(category: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prefix = format!(" {} ", category);
    let render_config = RenderConfig::default().with_prompt_prefix(
        Styled::new(prefix.as_str())
            .with_fg(Color::White)
            .with_bg(Color::DarkCyan),
    );

    let message = Text::new("")
        .with_render_config(render_config)
        .with_placeholder("enter commit message...")
        .prompt()?;

    let message = message.trim().to_string();
    if message.is_empty() {
        return Err("Commit message cannot be empty".into());
    }

    Ok(message)
}

pub fn prompt_description() -> Result<String, Box<dyn std::error::Error>> {
    let render_config =
        RenderConfig::default().with_prompt_prefix(Styled::new("    ").with_fg(Color::DarkGrey));

    let desc = Text::new("Description (optional):")
        .with_render_config(render_config)
        .with_default("")
        .prompt()?;

    Ok(desc)
}

/// Prompt for a commit message pre-filled with the last commit's message.
/// The user can edit it or press Enter to keep it unchanged.
pub fn prompt_amend_message(current: &str) -> Result<String, Box<dyn std::error::Error>> {
    let render_config = RenderConfig::default().with_prompt_prefix(
        Styled::new(" amend ")
            .with_fg(Color::White)
            .with_bg(Color::DarkYellow),
    );

    let message = Text::new("")
        .with_render_config(render_config)
        .with_initial_value(current)
        .prompt()?;

    let message = message.trim().to_string();
    if message.is_empty() {
        return Err("Commit message cannot be empty".into());
    }

    Ok(message)
}

pub fn prompt_release_tag(
    previous_tag: Option<&str>,
) -> Result<String, Box<dyn std::error::Error>> {
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
