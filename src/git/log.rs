use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};

use super::common::git_command;
use crate::error::Result;
use crate::style::{DIM, NAV_ARROWS, RESET};

pub fn log_graph() -> Result<()> {
    let output = git_command()
        .args([
            "log",
            "--color=always",
            "--graph",
            "--pretty=format:%C(yellow)%h%Creset %C(magenta)%ad%Creset %C(white)%<(30,trunc)%s %C(bold blue)%<(15,trunc)%an%Creset %C(auto)%d%Creset",
            "--date=short",
        ])
        .output()?;

    if output.status.success() {
        let log_output = String::from_utf8_lossy(&output.stdout).into_owned();
        if !log_output.trim().is_empty() {
            page_log(&log_output)?;
        }
        Ok(())
    } else {
        Err(format!(
            "Git log failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )
        .into())
    }
}

fn page_log(output: &str) -> Result<()> {
    let lines: Vec<&str> = output.lines().collect();
    let mut stdout = io::stdout();
    let _guard = TerminalGuard::enter(&mut stdout)?;

    let mut offset = 0usize;

    loop {
        render_page(&lines, offset, &mut stdout)?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Enter | KeyCode::Down | KeyCode::Char('j') => {
                    if offset + 1 < lines.len() {
                        offset += 1;
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    offset = offset.saturating_sub(1);
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn render_page(lines: &[&str], offset: usize, stdout: &mut io::Stdout) -> Result<usize> {
    let (_, rows) = terminal::size()?;
    let visible_lines = rows.saturating_sub(1).max(1) as usize;
    let end = (offset + visible_lines).min(lines.len());

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::All)
    )?;

    for line in &lines[offset..end] {
        queue!(stdout, Print(*line), Print("\r\n"))?;
    }

    let footer = if end >= lines.len() {
        format!("  {DIM}End of log · ↑/↓ move · q quit{RESET}")
    } else {
        format!("  {DIM}{NAV_ARROWS} move one commit · q quit{RESET}")
    };
    queue!(stdout, Print(footer))?;
    stdout.flush()?;

    Ok(visible_lines)
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter(stdout: &mut io::Stdout) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        if let Err(err) = execute!(stdout, cursor::Hide) {
            let _ = terminal::disable_raw_mode();
            return Err(err);
        }

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}
