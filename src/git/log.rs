use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
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
    let mut last_rendered_offset = usize::MAX;
    let mut last_size = terminal::size()?;
    let mut show_end_notice = false;
    let mut last_show_end_notice = false;

    loop {
        let size = terminal::size()?;
        let visible_lines = size.1.saturating_sub(1).max(1) as usize;
        let max_offset = lines.len().saturating_sub(visible_lines);
        if offset > max_offset {
            offset = max_offset;
        }

        if offset != last_rendered_offset
            || size != last_size
            || show_end_notice != last_show_end_notice
        {
            render_page(&lines, offset, show_end_notice, &mut stdout)?;
            last_rendered_offset = offset;
            last_size = size;
            last_show_end_notice = show_end_notice;
        }

        match event::read()? {
            Event::Key(key) => {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Enter | KeyCode::Down | KeyCode::Char('j') => {
                        if offset < max_offset {
                            offset += 1;
                            show_end_notice = false;
                        } else {
                            show_end_notice = true;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('k') => {
                        offset = offset.saturating_sub(1);
                        show_end_notice = false;
                    }
                    _ => {}
                }
            }
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    Ok(())
}

fn render_page(
    lines: &[&str],
    offset: usize,
    show_end_notice: bool,
    stdout: &mut io::Stdout,
) -> Result<()> {
    let (cols, rows) = terminal::size()?;
    let visible_lines = rows.saturating_sub(1).max(1) as usize;
    let end = (offset + visible_lines).min(lines.len());

    execute!(
        stdout,
        cursor::MoveTo(0, 0),
        terminal::Clear(ClearType::FromCursorDown)
    )?;

    for line in &lines[offset..end] {
        queue!(stdout, Print(clip(line, cols as usize)), Print("\r\n"))?;
    }

    let footer = if show_end_notice {
        format!("  {DIM}END{RESET}")
    } else if end >= lines.len() {
        format!("  {DIM}End of log · ↑/↓ move · q quit{RESET}")
    } else {
        format!("  {DIM}{NAV_ARROWS} move one commit · q quit{RESET}")
    };
    queue!(stdout, Print(footer))?;
    stdout.flush()?;

    Ok(())
}

/// Clip `line` to `cols` display cells, passing ANSI escapes through uncounted,
/// so wrapped lines can't overflow the screen and scroll the newest commits away.
fn clip(line: &str, cols: usize) -> String {
    let mut out = String::with_capacity(line.len());
    let mut width = 0;
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push(c);
            if chars.peek() == Some(&'[') {
                for e in chars.by_ref() {
                    out.push(e);
                    if e.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
            continue;
        }
        let w = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width + w > cols {
            out.push_str(RESET);
            break;
        }
        width += w;
        out.push(c);
    }
    out
}

struct TerminalGuard;

impl TerminalGuard {
    fn enter(stdout: &mut io::Stdout) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        if let Err(err) = execute!(stdout, EnterAlternateScreen, cursor::Hide) {
            let _ = terminal::disable_raw_mode();
            return Err(err);
        }

        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

#[cfg(test)]
mod tests {
    use super::clip;

    #[test]
    fn clip_ignores_ansi_and_truncates() {
        assert_eq!(clip("\x1b[33mabcdef\x1b[0m", 3), "\x1b[33mabc\x1b[0m");
        assert_eq!(clip("abc", 10), "abc");
    }
}
