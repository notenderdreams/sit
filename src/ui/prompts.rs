use inquire::Text;
use inquire::ui::{Color, RenderConfig, Styled};

use crate::error::Result;

pub fn prompt_message(category: &str) -> Result<String> {
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

fn render_input(
    buffer: &str,
    cursor_index: usize,
    start_col: u16,
    stdout: &mut std::io::Stdout,
) -> std::io::Result<()> {
    use crossterm::{cursor, queue, style::Print, terminal};
    use std::io::Write;

    let replacement = format!(" {}↵{} ", crate::style::GREY, crate::style::RESET);
    let display_str = buffer.replace('\n', &replacement);

    let mut visual_cursor = 0;
    for (i, c) in buffer.chars().enumerate() {
        if i == cursor_index {
            break;
        }
        if c == '\n' {
            visual_cursor += 3;
        } else {
            visual_cursor += 1;
        }
    }

    queue!(
        stdout,
        cursor::MoveToColumn(start_col),
        terminal::Clear(terminal::ClearType::UntilNewLine),
        Print(&display_str),
        cursor::MoveToColumn(start_col + visual_cursor as u16)
    )?;
    stdout.flush()
}

pub fn prompt_description() -> Result<String> {
    use crossterm::{
        cursor,
        event::{self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEventKind},
        execute, queue,
        style::Print,
    };
    use std::io::Write;

    super::terminal::run_with_terminal(|stdout| {
        // Show cursor
        execute!(stdout, cursor::Show)?;

        // Enable bracketed paste
        let _ = execute!(stdout, EnableBracketedPaste);

        // Print prompt
        queue!(
            stdout,
            Print(format!(
                "{}    {}Description (optional): ",
                crate::style::GREY,
                crate::style::RESET
            ))
        )?;
        stdout.flush()?;

        let start_col = cursor::position().map(|(col, _)| col).unwrap_or(28);

        let mut buffer = String::new();
        let mut cursor_index = 0; // index in characters

        let result = loop {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    match key.code {
                        KeyCode::Enter | KeyCode::Char('\n') | KeyCode::Char('\r') => {
                            queue!(stdout, Print("\r\n"))?;
                            stdout.flush()?;
                            break Ok(buffer);
                        }
                        KeyCode::Esc => {
                            queue!(stdout, Print("\r\n"))?;
                            stdout.flush()?;
                            break Ok(String::new());
                        }
                        KeyCode::Char('c')
                            if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
                        {
                            queue!(stdout, Print("\r\n"))?;
                            stdout.flush()?;
                            break Err("Cancelled by user".into());
                        }
                        KeyCode::Char(c) => {
                            let byte_idx = buffer
                                .char_indices()
                                .map(|(i, _)| i)
                                .nth(cursor_index)
                                .unwrap_or(buffer.len());
                            buffer.insert(byte_idx, c);
                            cursor_index += 1;
                            render_input(&buffer, cursor_index, start_col, stdout)?;
                        }
                        KeyCode::Backspace => {
                            if cursor_index > 0 {
                                cursor_index -= 1;
                                let byte_idx = buffer
                                    .char_indices()
                                    .map(|(i, _)| i)
                                    .nth(cursor_index)
                                    .unwrap();
                                buffer.remove(byte_idx);
                                render_input(&buffer, cursor_index, start_col, stdout)?;
                            }
                        }
                        KeyCode::Delete => {
                            if cursor_index < buffer.chars().count() {
                                let byte_idx = buffer
                                    .char_indices()
                                    .map(|(i, _)| i)
                                    .nth(cursor_index)
                                    .unwrap();
                                buffer.remove(byte_idx);
                                render_input(&buffer, cursor_index, start_col, stdout)?;
                            }
                        }
                        KeyCode::Left => {
                            if cursor_index > 0 {
                                cursor_index -= 1;
                                render_input(&buffer, cursor_index, start_col, stdout)?;
                            }
                        }
                        KeyCode::Right => {
                            if cursor_index < buffer.chars().count() {
                                cursor_index += 1;
                                render_input(&buffer, cursor_index, start_col, stdout)?;
                            }
                        }
                        KeyCode::Home => {
                            cursor_index = 0;
                            render_input(&buffer, cursor_index, start_col, stdout)?;
                        }
                        KeyCode::End => {
                            cursor_index = buffer.chars().count();
                            render_input(&buffer, cursor_index, start_col, stdout)?;
                        }
                        _ => {}
                    }
                }
                Event::Paste(pasted_text) => {
                    let cleaned = pasted_text.replace("\r\n", "\n").replace('\r', "\n");
                    let byte_idx = buffer
                        .char_indices()
                        .map(|(i, _)| i)
                        .nth(cursor_index)
                        .unwrap_or(buffer.len());
                    buffer.insert_str(byte_idx, &cleaned);
                    cursor_index += cleaned.chars().count();
                    render_input(&buffer, cursor_index, start_col, stdout)?;
                }
                _ => {}
            }
        };

        let _ = execute!(stdout, DisableBracketedPaste);
        result
    })
}

/// Prompt for a commit message pre-filled with the last commit's message.
/// The user can edit it or press Enter to keep it unchanged.
pub fn prompt_amend_message(current: &str) -> Result<String> {
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
