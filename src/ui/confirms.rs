use std::io::{self, Write};

use crate::style::{BOLD, CYAN, DIM, RESET, TREE_LAST, TREE_MID};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    queue,
    style::Print,
};

use super::terminal::{clear_lines, run_with_terminal};
use crate::error::Result;

/// Show a commit preview and ask the user to confirm with y/N.
/// Returns `true` if the user pressed y/Y, `false` on n/N/Esc/any other key.
pub fn confirm_commit(subject: &str, emoji: &str, files: &[String]) -> Result<bool> {
    run_with_terminal(|stdout| confirm_commit_loop(subject, emoji, files, stdout))
}

fn confirm_commit_loop(
    subject: &str,
    emoji: &str,
    files: &[String],
    stdout: &mut io::Stdout,
) -> Result<bool> {
    let line_count = render_commit_preview(subject, emoji, files, stdout)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    clear_lines(line_count, stdout)?;
                    return Ok(true);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    clear_lines(line_count, stdout)?;
                    return Ok(false);
                }
                _ => {}
            }
        }
    }
}

fn render_commit_preview(
    subject: &str,
    emoji: &str,
    files: &[String],
    stdout: &mut io::Stdout,
) -> io::Result<usize> {
    queue!(stdout, Print("\r\n"))?;
    queue!(stdout, Print(format!("  {BOLD}Preview commit:{RESET}\r\n")))?;
    queue!(stdout, Print("\r\n"))?;

    let trimmed_emoji = emoji.trim();
    if trimmed_emoji.is_empty() {
        queue!(
            stdout,
            Print(format!("    {CYAN}{BOLD}{subject}{RESET}\r\n"))
        )?;
    } else {
        queue!(
            stdout,
            Print(format!("    {emoji}{CYAN}{BOLD}{subject}{RESET}\r\n"))
        )?;
    }

    queue!(stdout, Print("\r\n"))?;
    queue!(stdout, Print(format!("  {BOLD}Files:{RESET}\r\n")))?;

    let last = files.len().saturating_sub(1);
    for (i, f) in files.iter().enumerate() {
        let branch = if i == last { TREE_LAST } else { TREE_MID };
        queue!(stdout, Print(format!("    {DIM}{branch}{RESET} {f}\r\n")))?;
    }

    queue!(
        stdout,
        Print(format!("\r\n  {BOLD}Confirm?{RESET} {DIM}[Y/n]{RESET}  "))
    )?;
    stdout.flush()?;

    Ok(7 + files.len())
}

/// Ask "Push now? [y/N]" with a single keypress.
/// Returns `true` if the user pressed y/Y.
pub fn confirm_push() -> Result<bool> {
    confirm_simple("\r\n  Push now?", true)
}

pub fn confirm_create_branch(name: &str) -> Result<bool> {
    let prompt = format!("\r\n  Create branch {CYAN}{name}{RESET}?");
    confirm_simple(&prompt, false)
}

/// Ask "Confirm undo? [y/N]" with a single keypress.
pub fn confirm_undo() -> Result<bool> {
    confirm_simple("  Confirm undo?", false)
}

fn confirm_simple(prompt: &str, enter_yes: bool) -> Result<bool> {
    run_with_terminal(|stdout| {
        queue!(
            stdout,
            Print(format!("{BOLD}{prompt}{RESET} {DIM}[y/N]{RESET}  "))
        )?;
        stdout.flush()?;

        let confirmed = loop {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                let yes = matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'));
                if enter_yes && key.code == KeyCode::Enter {
                    break true;
                }
                break yes;
            }
        };

        clear_lines(1, stdout)?;
        Ok(confirmed)
    })
}
