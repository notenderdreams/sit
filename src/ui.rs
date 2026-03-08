use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};
use inquire::Text;
use inquire::ui::{Color, RenderConfig, Styled};

use crate::categories::Category;
use crate::config::Module;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BG_SELECT: &str = "\x1b[48;5;236m";
const CYAN: &str = "\x1b[36m";

pub fn select_category(
    categories: &[Category],
    show_emoji: bool,
) -> Result<&str, Box<dyn std::error::Error>> {
    let mut cursor_pos: usize = 0;
    let mut filter = String::new();
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = category_loop(
        categories,
        show_emoji,
        &mut cursor_pos,
        &mut filter,
        &mut stdout,
    );

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn filtered_indices(categories: &[Category], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        (0..categories.len()).collect()
    } else {
        let f = filter.to_lowercase();
        categories
            .iter()
            .enumerate()
            .filter(|(_, c)| {
                c.name.to_lowercase().contains(&f) || c.description.to_lowercase().contains(&f)
            })
            .map(|(i, _)| i)
            .collect()
    }
}

fn category_loop<'a>(
    categories: &'a [Category],
    show_emoji: bool,
    cursor_pos: &mut usize,
    filter: &mut String,
    stdout: &mut io::Stdout,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let mut last_height: usize = 0;
    last_height = render_categories(
        categories,
        show_emoji,
        &filtered_indices(categories, filter),
        *cursor_pos,
        filter,
        last_height,
        stdout,
    )?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::BackTab => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    let vis = filtered_indices(categories, filter);
                    if *cursor_pos < vis.len().saturating_sub(1) {
                        *cursor_pos += 1;
                    }
                }
                KeyCode::Backspace => {
                    filter.pop();
                    *cursor_pos = 0;
                }
                KeyCode::Char(ch) => {
                    filter.push(ch);
                    *cursor_pos = 0;
                }
                KeyCode::Enter => {
                    let vis = filtered_indices(categories, filter);
                    if let Some(&idx) = vis.get(*cursor_pos) {
                        let cat = &categories[idx];
                        clear_lines(last_height, stdout)?;
                        // Print selected
                        if show_emoji {
                            queue!(
                                stdout,
                                Print(format!(
                                    "\r\n  {BOLD}  Type{RESET}  {} {CYAN}{}{RESET} {DIM}{}{RESET}\r\n\r\n",
                                    cat.emoji, cat.name, cat.description
                                ))
                            )?;
                        } else {
                            queue!(
                                stdout,
                                Print(format!(
                                    "\r\n  {BOLD}  Type{RESET}  {CYAN}{}{RESET} {DIM}{}{RESET}\r\n\r\n",
                                    cat.name, cat.description
                                ))
                            )?;
                        }
                        stdout.flush()?;
                        return Ok(&cat.name);
                    }
                }
                KeyCode::Esc => {
                    clear_lines(last_height, stdout)?;
                    return Err("Cancelled".into());
                }
                _ => {}
            }

            let vis = filtered_indices(categories, filter);
            if *cursor_pos >= vis.len() {
                *cursor_pos = vis.len().saturating_sub(1);
            }
            last_height = render_categories(
                categories,
                show_emoji,
                &vis,
                *cursor_pos,
                filter,
                last_height,
                stdout,
            )?;
        }
    }
}

fn render_categories(
    categories: &[Category],
    show_emoji: bool,
    visible: &[usize],
    cursor_pos: usize,
    filter: &str,
    prev_height: usize,
    stdout: &mut io::Stdout,
) -> io::Result<usize> {
    // Clear only the lines we rendered last time
    if prev_height > 0 {
        queue!(stdout, cursor::MoveToColumn(0))?;
        for _ in 0..prev_height {
            queue!(
                stdout,
                terminal::Clear(ClearType::CurrentLine),
                cursor::MoveUp(1)
            )?;
        }
        queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    }

    // Header + filter
    if filter.is_empty() {
        queue!(
            stdout,
            Print(format!(
                "\r\n  {BOLD}  Commit type{RESET} {DIM}(type to filter){RESET}\r\n\r\n"
            ))
        )?;
    } else {
        queue!(
            stdout,
            Print(format!(
                "\r\n  {BOLD}  Commit type{RESET}  {DIM}/{RESET}{CYAN}{filter}{RESET}\r\n\r\n"
            ))
        )?;
    }

    let max_name = categories.iter().map(|c| c.name.len()).max().unwrap_or(8);
    let len = visible.len();

    if len == 0 {
        queue!(stdout, Print(format!("    {DIM}no matches{RESET}\r\n")))?;
    } else {
        for (vi, &ci) in visible.iter().enumerate() {
            let cat = &categories[ci];
            let is_cursor = vi == cursor_pos;

            let bg = if is_cursor { BG_SELECT } else { "" };
            let end_bg = if is_cursor { RESET } else { "" };
            let pointer = if is_cursor {
                format!("{CYAN}›{RESET}")
            } else {
                " ".to_string()
            };

            if show_emoji {
                queue!(
                    stdout,
                    Print(format!(
                        "  {bg} {pointer}  {}  {BOLD}{:<width$}{RESET}{bg}  {DIM}{}{end_bg}{RESET}\r\n",
                        cat.emoji,
                        cat.name,
                        cat.description,
                        width = max_name
                    ))
                )?;
            } else {
                queue!(
                    stdout,
                    Print(format!(
                        "  {bg} {pointer}  {BOLD}{:<width$}{RESET}{bg}  {DIM}{}{end_bg}{RESET}\r\n",
                        cat.name,
                        cat.description,
                        width = max_name
                    ))
                )?;
            }
        }
    }

    // Help
    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}↑↓{RESET} move  {DIM}type{RESET} filter  {DIM}enter{RESET} select  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()?;

    // leading \r\n(1) + header \r\n(1) + blank \r\n(1) + items + help \r\n(1) = items + 4
    let item_lines = if visible.is_empty() { 1 } else { visible.len() };
    Ok(item_lines + 4)
}

/// Clear exactly `n` rendered lines from the terminal.
fn clear_lines(n: usize, stdout: &mut io::Stdout) -> io::Result<()> {
    if n == 0 {
        return Ok(());
    }
    queue!(stdout, cursor::MoveToColumn(0))?;
    for _ in 0..n {
        queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveUp(1)
        )?;
    }
    queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    stdout.flush()
}

pub fn select_module(modules: &[Module]) -> Result<Option<&str>, Box<dyn std::error::Error>> {
    let mut cursor_pos: usize = 0;
    let mut filter = String::new();
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = module_loop(modules, &mut cursor_pos, &mut filter, &mut stdout);

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
}

fn filtered_module_indices(modules: &[Module], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        (0..modules.len()).collect()
    } else {
        let f = filter.to_lowercase();
        modules
            .iter()
            .enumerate()
            .filter(|(_, m)| {
                m.name.to_lowercase().contains(&f) || m.description.to_lowercase().contains(&f)
            })
            .map(|(i, _)| i)
            .collect()
    }
}

fn module_loop<'a>(
    modules: &'a [Module],
    cursor_pos: &mut usize,
    filter: &mut String,
    stdout: &mut io::Stdout,
) -> Result<Option<&'a str>, Box<dyn std::error::Error>> {
    let mut last_height: usize = 0;
    last_height = render_modules(
        modules,
        &filtered_module_indices(modules, filter),
        *cursor_pos,
        filter,
        last_height,
        stdout,
    )?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let visible = filtered_module_indices(modules, filter);
            let total_items = visible.len() + 1; // +1 for skip row

            match key.code {
                KeyCode::Up | KeyCode::BackTab => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    if *cursor_pos < total_items.saturating_sub(1) {
                        *cursor_pos += 1;
                    }
                }
                KeyCode::Backspace => {
                    filter.pop();
                    *cursor_pos = 0;
                }
                KeyCode::Char(ch) => {
                    filter.push(ch);
                    *cursor_pos = 0;
                }
                KeyCode::Enter => {
                    let vis = filtered_module_indices(modules, filter);
                    clear_lines(last_height, stdout)?;

                    // Last item = skip (no module)
                    if *cursor_pos >= vis.len() {
                        queue!(
                            stdout,
                            Print(format!(
                                "\r\n  {BOLD}  Module{RESET}  {DIM}none{RESET}\r\n\r\n"
                            ))
                        )?;
                        stdout.flush()?;
                        return Ok(None);
                    }

                    let idx = vis[*cursor_pos];
                    let m = &modules[idx];
                    queue!(
                        stdout,
                        Print(format!(
                            "\r\n  {BOLD}  Module{RESET}  {CYAN}{}{RESET} {DIM}{}{RESET}\r\n\r\n",
                            m.name, m.description
                        ))
                    )?;
                    stdout.flush()?;
                    return Ok(Some(&m.name));
                }
                KeyCode::Esc => {
                    clear_lines(last_height, stdout)?;
                    return Err("Cancelled".into());
                }
                _ => {}
            }

            let vis = filtered_module_indices(modules, filter);
            let total_items = vis.len() + 1;
            if *cursor_pos >= total_items {
                *cursor_pos = total_items.saturating_sub(1);
            }
            last_height = render_modules(modules, &vis, *cursor_pos, filter, last_height, stdout)?;
        }
    }
}

fn render_modules(
    modules: &[Module],
    visible: &[usize],
    cursor_pos: usize,
    filter: &str,
    prev_height: usize,
    stdout: &mut io::Stdout,
) -> io::Result<usize> {
    if prev_height > 0 {
        queue!(stdout, cursor::MoveToColumn(0))?;
        for _ in 0..prev_height {
            queue!(
                stdout,
                terminal::Clear(ClearType::CurrentLine),
                cursor::MoveUp(1)
            )?;
        }
        queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    }

    if filter.is_empty() {
        queue!(
            stdout,
            Print(format!(
                "\r\n  {BOLD}  Module{RESET} {DIM}(type to filter){RESET}\r\n\r\n"
            ))
        )?;
    } else {
        queue!(
            stdout,
            Print(format!(
                "\r\n  {BOLD}  Module{RESET}  {DIM}/{RESET}{CYAN}{filter}{RESET}\r\n\r\n"
            ))
        )?;
    }

    let max_name = modules
        .iter()
        .map(|m| m.name.len())
        .max()
        .unwrap_or(4)
        .max(4); // at least "none"

    if visible.is_empty() && filter.is_empty() {
        // shouldn't happen, but safety
    } else {
        for (vi, &mi) in visible.iter().enumerate() {
            let m = &modules[mi];
            let is_cursor = vi == cursor_pos;
            let bg = if is_cursor { BG_SELECT } else { "" };
            let end_bg = if is_cursor { RESET } else { "" };
            let pointer = if is_cursor {
                format!("{CYAN}\u{203a}{RESET}")
            } else {
                " ".to_string()
            };

            queue!(
                stdout,
                Print(format!(
                    "  {bg} {pointer}  {BOLD}{:<width$}{RESET}{bg}  {DIM}{}{end_bg}{RESET}\r\n",
                    m.name,
                    m.description,
                    width = max_name
                ))
            )?;
        }

        // "none" / skip row
        let skip_idx = visible.len();
        let is_cursor = cursor_pos == skip_idx;
        let bg = if is_cursor { BG_SELECT } else { "" };
        let end_bg = if is_cursor { RESET } else { "" };
        let pointer = if is_cursor {
            format!("{CYAN}\u{203a}{RESET}")
        } else {
            " ".to_string()
        };
        queue!(
            stdout,
            Print(format!(
                "  {bg} {pointer}  {DIM}{:<width$}  skip{end_bg}{RESET}\r\n",
                "none",
                width = max_name
            ))
        )?;
    }

    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}\u{2191}\u{2193}{RESET} move  {DIM}type{RESET} filter  {DIM}enter{RESET} select  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()?;

    // header(1) + blank(1) + items + skip(1) + blank(1) + help(1) = visible.len() + 4
    let item_lines = if visible.is_empty() { 0 } else { visible.len() };
    Ok(item_lines + 1 + 4) // +1 for skip row
}

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

pub fn print_success(commit_msg: &str) {
    crate::print::blank();
    crate::print::success_with_details("Committed", commit_msg);
    crate::print::blank();
}

// ── Confirm prompts ──────────────────────────────────────────────────────────

/// Show a commit preview and ask the user to confirm with y/N.
/// Returns `true` if the user pressed y/Y, `false` on n/N/Esc/any other key.
pub fn confirm_commit(
    subject: &str,
    emoji: &str,
    files: &[String],
) -> Result<bool, Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = confirm_commit_loop(subject, emoji, files, &mut stdout);

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    result
}

fn confirm_commit_loop(
    subject: &str,
    emoji: &str,
    files: &[String],
    stdout: &mut io::Stdout,
) -> Result<bool, Box<dyn std::error::Error>> {
    let line_count = render_commit_preview(subject, emoji, files, stdout)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let confirmed = matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'));
            // On any decision key (or any key at all), clear preview and return
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') => {
                    clear_lines(line_count, stdout)?;
                    return Ok(confirmed);
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc | KeyCode::Enter => {
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
    // blank line
    queue!(stdout, Print("\r\n"))?;
    // header
    queue!(stdout, Print(format!("  {BOLD}Preview commit:{RESET}\r\n")))?;
    // blank
    queue!(stdout, Print("\r\n"))?;

    // commit subject with emoji
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

    // blank
    queue!(stdout, Print("\r\n"))?;
    // files header
    queue!(stdout, Print(format!("  {BOLD}Files:{RESET}\r\n")))?;

    let last = files.len().saturating_sub(1);
    for (i, f) in files.iter().enumerate() {
        let branch = if i == last { "└──" } else { "├──" };
        queue!(stdout, Print(format!("    {DIM}{branch}{RESET} {f}\r\n")))?;
    }

    // blank + confirm prompt (no trailing \r\n — cursor stays on this line)
    queue!(
        stdout,
        Print(format!("\r\n  {BOLD}Confirm?{RESET} {DIM}[y/N]{RESET}  "))
    )?;
    stdout.flush()?;

    // Count of \r\n emitted:
    //   1 (leading blank) + 1 (header) + 1 (blank) + 1 (subject) + 1 (blank)
    //   + 1 (Files:) + files.len() (each file) + 1 (leading \r\n of Confirm line)
    // = 7 + files.len()
    Ok(7 + files.len())
}

/// Ask "Push now? [y/N]" with a single keypress.
/// Returns `true` if the user pressed y/Y.
pub fn confirm_push() -> Result<bool, Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    // Print prompt (1 leading \r\n → 1 \r\n total, cursor on prompt line)
    queue!(
        stdout,
        Print(format!("\r\n  {BOLD}Push now?{RESET} {DIM}[y/N]{RESET}  "))
    )?;
    stdout.flush()?;

    let confirmed = loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            break matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'));
        }
    };

    // Clear the 2 lines we drew (blank + prompt line)
    clear_lines(1, &mut stdout)?;

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(confirmed)
}

/// Ask "Confirm undo? [y/N]" with a single keypress.
pub fn confirm_undo() -> Result<bool, Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    queue!(
        stdout,
        Print(format!("  {BOLD}Confirm undo?{RESET} {DIM}[y/N]{RESET}  "))
    )?;
    stdout.flush()?;

    let confirmed = loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            break matches!(key.code, KeyCode::Char('y') | KeyCode::Char('Y'));
        }
    };

    clear_lines(1, &mut stdout)?;

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;
    Ok(confirmed)
}

pub fn print_error(msg: &str) {
    crate::print::blank();
    crate::print::error(msg);
    crate::print::blank();
}
