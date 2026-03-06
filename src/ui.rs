use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};
use inquire::Text;
use inquire::ui::{Color, RenderConfig, Styled};

use crate::categories::Category;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BG_SELECT: &str = "\x1b[48;5;236m";
const CYAN: &str = "\x1b[36m";

pub fn select_category<'a>(
    categories: &'a [Category],
    show_emoji: bool,
) -> Result<&'a str, Box<dyn std::error::Error>> {
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
    render_categories(
        categories,
        show_emoji,
        &filtered_indices(categories, filter),
        *cursor_pos,
        filter,
        stdout,
    )?;

    loop {
        if let Event::Key(key) = event::read()? {
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
                        clear_category_display(categories, stdout)?;
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
                    clear_category_display(categories, stdout)?;
                    return Err("Cancelled".into());
                }
                _ => {}
            }

            let vis = filtered_indices(categories, filter);
            if *cursor_pos >= vis.len() {
                *cursor_pos = vis.len().saturating_sub(1);
            }
            render_categories(categories, show_emoji, &vis, *cursor_pos, filter, stdout)?;
        }
    }
}

fn render_categories(
    categories: &[Category],
    show_emoji: bool,
    visible: &[usize],
    cursor_pos: usize,
    filter: &str,
    stdout: &mut io::Stdout,
) -> io::Result<()> {
    let total_lines = categories.len() + 5;
    queue!(stdout, cursor::MoveToColumn(0))?;
    for _ in 0..total_lines {
        queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveUp(1)
        )?;
    }
    queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;

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

    stdout.flush()
}

fn clear_category_display(categories: &[Category], stdout: &mut io::Stdout) -> io::Result<()> {
    let total_lines = categories.len() + 5;
    queue!(stdout, cursor::MoveToColumn(0))?;
    for _ in 0..total_lines {
        queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveUp(1)
        )?;
    }
    queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    stdout.flush()
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
