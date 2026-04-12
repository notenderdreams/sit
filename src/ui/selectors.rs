use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    queue,
    style::Print,
    terminal::{self, ClearType},
};

use crate::categories::Category;
use crate::config::Module;
use crate::error::Result;
use crate::git::Branch;
use crate::style::{BG_SELECT, BOLD, CYAN, DIM, NAV_ARROWS, POINTER, RESET};

use super::terminal::{clear_lines, run_with_terminal};

pub fn select_category(categories: &[Category]) -> Result<&str> {
    run_with_terminal(|stdout| {
        let mut cursor_pos: usize = 0;
        let mut filter = String::new();
        category_loop(categories, &mut cursor_pos, &mut filter, stdout)
    })
}

pub fn select_module(modules: &[Module]) -> Result<Option<&str>> {
    run_with_terminal(|stdout| {
        let mut cursor_pos: usize = 0;
        let mut filter = String::new();
        module_loop(modules, &mut cursor_pos, &mut filter, stdout)
    })
}

pub fn select_branch(branches: &[Branch]) -> Result<String> {
    run_with_terminal(|stdout| {
        let mut cursor_pos: usize = 0;
        let mut filter = String::new();
        branch_loop(branches, &mut cursor_pos, &mut filter, stdout)
    })
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
    cursor_pos: &mut usize,
    filter: &mut String,
    stdout: &mut io::Stdout,
) -> Result<&'a str> {
    let mut last_height = render_categories(
        categories,
        &filtered_indices(categories, filter),
        *cursor_pos,
        filter,
        0,
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
                        queue!(
                            stdout,
                            Print(format!(
                                "\r\n  {BOLD}  Type{RESET}  {CYAN}{}{RESET} {DIM}{}{RESET}\r\n\r\n",
                                cat.name, cat.description
                            ))
                        )?;
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
            last_height =
                render_categories(categories, &vis, *cursor_pos, filter, last_height, stdout)?;
        }
    }
}

fn render_categories(
    categories: &[Category],
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

    if visible.is_empty() {
        queue!(stdout, Print(format!("    {DIM}no matches{RESET}\r\n")))?;
    } else {
        for (vi, &ci) in visible.iter().enumerate() {
            let cat = &categories[ci];
            let is_cursor = vi == cursor_pos;

            let bg = if is_cursor { BG_SELECT } else { "" };
            let end_bg = if is_cursor { RESET } else { "" };
            let pointer = if is_cursor {
                format!("{CYAN}{POINTER}{RESET}")
            } else {
                " ".to_string()
            };

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

    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}{NAV_ARROWS}{RESET} move  {DIM}type{RESET} filter  {DIM}enter{RESET} select  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()?;
    let item_lines = if visible.is_empty() { 1 } else { visible.len() };
    Ok(item_lines + 4)
}

fn filtered_branch_indices(branches: &[Branch], filter: &str) -> Vec<usize> {
    if filter.is_empty() {
        (0..branches.len()).collect()
    } else {
        let f = filter.to_lowercase();
        branches
            .iter()
            .enumerate()
            .filter(|(_, b)| b.name.to_lowercase().contains(&f))
            .map(|(i, _)| i)
            .collect()
    }
}

fn has_exact_branch_match(branches: &[Branch], value: &str) -> bool {
    branches.iter().any(|b| b.name == value)
}

fn branch_loop(
    branches: &[Branch],
    cursor_pos: &mut usize,
    filter: &mut String,
    stdout: &mut io::Stdout,
) -> Result<String> {
    let mut vis = filtered_branch_indices(branches, filter);
    let mut can_create = !filter.trim().is_empty() && !has_exact_branch_match(branches, filter);

    let mut last_height =
        render_branches(branches, &vis, *cursor_pos, filter, can_create, 0, stdout)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            let total_items = vis.len() + usize::from(can_create);

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
                    clear_lines(last_height, stdout)?;
                    if let Some(&idx) = vis.get(*cursor_pos) {
                        let selected = &branches[idx].name;
                        queue!(
                            stdout,
                            Print(format!(
                                "\r\n  {BOLD}  Branch{RESET}  {CYAN}{selected}{RESET}\r\n\r\n"
                            ))
                        )?;
                        stdout.flush()?;
                        return Ok(selected.clone());
                    }

                    if can_create {
                        let new_branch = filter.trim().to_owned();
                        queue!(
                            stdout,
                            Print(format!(
                                "\r\n  {BOLD}  Branch{RESET}  {CYAN}{new_branch}{RESET} {DIM}(new){RESET}\r\n\r\n"
                            ))
                        )?;
                        stdout.flush()?;
                        return Ok(new_branch);
                    }
                }
                KeyCode::Esc => {
                    clear_lines(last_height, stdout)?;
                    return Err("Cancelled".into());
                }
                _ => {}
            }

            vis = filtered_branch_indices(branches, filter);
            can_create = !filter.trim().is_empty() && !has_exact_branch_match(branches, filter);

            let total_items = vis.len() + usize::from(can_create);
            if *cursor_pos >= total_items {
                *cursor_pos = total_items.saturating_sub(1);
            }

            last_height = render_branches(
                branches,
                &vis,
                *cursor_pos,
                filter,
                can_create,
                last_height,
                stdout,
            )?;
        }
    }
}

fn render_branches(
    branches: &[Branch],
    visible: &[usize],
    cursor_pos: usize,
    filter: &str,
    can_create: bool,
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
                "\r\n  {BOLD}  Branch{RESET} {DIM}(type to search or create){RESET}\r\n\r\n"
            ))
        )?;
    } else {
        queue!(
            stdout,
            Print(format!(
                "\r\n  {BOLD}  Branch{RESET}  {DIM}/{RESET}{CYAN}{filter}{RESET}\r\n\r\n"
            ))
        )?;
    }

    let max_name = branches
        .iter()
        .map(|b| b.name.len())
        .max()
        .unwrap_or(6)
        .max(6);
    let mut item_lines = 0usize;

    for (vi, &bi) in visible.iter().enumerate() {
        let branch = &branches[bi];
        let is_cursor = vi == cursor_pos;
        let bg = if is_cursor { BG_SELECT } else { "" };
        let end_bg = if is_cursor { RESET } else { "" };
        let pointer = if is_cursor {
            format!("{CYAN}{POINTER}{RESET}")
        } else {
            " ".to_string()
        };
        let current = if branch.is_current {
            format!("{DIM}(current){RESET}")
        } else {
            String::new()
        };

        queue!(
            stdout,
            Print(format!(
                "  {bg} {pointer}  {BOLD}{:<width$}{RESET}{bg}  {current}{end_bg}{RESET}\r\n",
                branch.name,
                width = max_name
            ))
        )?;
        item_lines += 1;
    }

    if can_create {
        let create_idx = visible.len();
        let is_cursor = cursor_pos == create_idx;
        let bg = if is_cursor { BG_SELECT } else { "" };
        let end_bg = if is_cursor { RESET } else { "" };
        let pointer = if is_cursor {
            format!("{CYAN}{POINTER}{RESET}")
        } else {
            " ".to_string()
        };
        queue!(
            stdout,
            Print(format!(
                "  {bg} {pointer}  {DIM}create branch:{RESET} {CYAN}{}{end_bg}{RESET}\r\n",
                filter.trim()
            ))
        )?;
        item_lines += 1;
    }

    if item_lines == 0 {
        queue!(stdout, Print(format!("    {DIM}no matches{RESET}\r\n")))?;
        item_lines = 1;
    }

    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}{NAV_ARROWS}{RESET} move  {DIM}type{RESET} search  {DIM}enter{RESET} select  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()?;
    Ok(item_lines + 4)
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
) -> Result<Option<&'a str>> {
    let mut last_height = render_modules(
        modules,
        &filtered_module_indices(modules, filter),
        *cursor_pos,
        filter,
        0,
        stdout,
    )?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            let visible = filtered_module_indices(modules, filter);
            let total_items = visible.len() + 1;

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
        .max(4);

    for (vi, &mi) in visible.iter().enumerate() {
        let m = &modules[mi];
        let is_cursor = vi == cursor_pos;
        let bg = if is_cursor { BG_SELECT } else { "" };
        let end_bg = if is_cursor { RESET } else { "" };
        let pointer = if is_cursor {
            format!("{CYAN}{POINTER}{RESET}")
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

    let skip_idx = visible.len();
    let is_cursor = cursor_pos == skip_idx;
    let bg = if is_cursor { BG_SELECT } else { "" };
    let end_bg = if is_cursor { RESET } else { "" };
    let pointer = if is_cursor {
        format!("{CYAN}{POINTER}{RESET}")
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

    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}{NAV_ARROWS}{RESET} move  {DIM}type{RESET} filter  {DIM}enter{RESET} select  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()?;

    let item_lines = visible.len();
    Ok(item_lines + 1 + 4)
}
