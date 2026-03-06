use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};

use crate::git::{FileChange, FileStatus};

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const BG_SELECT: &str = "\x1b[48;5;236m";

struct Item {
    change: FileChange,
    selected: bool,
}

pub fn pick_files(changes: Vec<FileChange>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    if changes.is_empty() {
        return Ok(vec![]);
    }

    let mut items: Vec<Item> = changes
        .into_iter()
        .map(|c| {
            let selected = c.status != FileStatus::Untracked;
            Item {
                change: c,
                selected,
            }
        })
        .collect();

    // Sort: modified, added, deleted, renamed, untracked
    items.sort_by_key(|i| i.change.status.order());

    let mut cursor_pos: usize = 0;
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = run_loop(&mut items, &mut cursor_pos, &mut stdout);

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    match result {
        Ok(true) => {
            let selected: Vec<String> = items
                .into_iter()
                .filter(|i| i.selected)
                .map(|i| i.change.path)
                .collect();
            Ok(selected)
        }
        Ok(false) => Err("Cancelled".into()),
        Err(e) => Err(e.into()),
    }
}

fn run_loop(
    items: &mut [Item],
    cursor_pos: &mut usize,
    stdout: &mut io::Stdout,
) -> io::Result<bool> {
    render(items, *cursor_pos, stdout)?;

    loop {
        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if *cursor_pos > 0 {
                        *cursor_pos -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if *cursor_pos < items.len() - 1 {
                        *cursor_pos += 1;
                    }
                }
                KeyCode::Char(' ') => {
                    items[*cursor_pos].selected = !items[*cursor_pos].selected;
                }
                KeyCode::Char('a') => {
                    let all_on = items.iter().all(|i| i.selected);
                    for item in items.iter_mut() {
                        item.selected = !all_on;
                    }
                }
                KeyCode::Enter => {
                    clear_display(items.len(), stdout)?;
                    let selected: Vec<&Item> = items.iter().filter(|i| i.selected).collect();
                    let count = selected.len();
                    queue!(
                        stdout,
                        Print(format!(
                            "\r\n  {BOLD}  Files{RESET} {DIM}({count} selected){RESET}\r\n"
                        ))
                    )?;
                    for (i, item) in selected.iter().enumerate() {
                        let c = item.change.status.color_code();
                        let icon = item.change.status.icon();
                        let conn = if i == count - 1 {
                            "└── "
                        } else {
                            "├── "
                        };
                        queue!(
                            stdout,
                            Print(format!(
                                "    {DIM}{conn}{RESET}{c}{icon} {}{RESET}\r\n",
                                item.change.path
                            ))
                        )?;
                    }
                    queue!(stdout, Print("\r\n"))?;
                    stdout.flush()?;
                    return Ok(true);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    clear_display(items.len(), stdout)?;
                    return Ok(false);
                }
                _ => {}
            }
            render(items, *cursor_pos, stdout)?;
        }
    }
}

fn render(items: &[Item], cursor_pos: usize, stdout: &mut io::Stdout) -> io::Result<()> {
    let total_lines = items.len() + 4; // header + blank + items + help
    queue!(stdout, cursor::MoveToColumn(0))?;
    for _ in 0..total_lines {
        queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveUp(1)
        )?;
    }
    queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;

    let selected_count = items.iter().filter(|i| i.selected).count();

    // Header
    queue!(
        stdout,
        Print(format!(
            "\r\n  {BOLD}  Changes{RESET} {DIM}({selected_count}/{} selected){RESET}\r\n\r\n",
            items.len()
        ))
    )?;

    // Items as tree
    let len = items.len();
    for (i, item) in items.iter().enumerate() {
        let is_cursor = i == cursor_pos;
        let is_last = i == len - 1;
        let c = item.change.status.color_code();
        let icon = item.change.status.icon();

        let connector = if is_last { "└── " } else { "├── " };

        let checkbox = if item.selected {
            format!("{c}◉{RESET}")
        } else {
            format!("{DIM}○{RESET}")
        };

        let bg = if is_cursor { BG_SELECT } else { "" };
        let end = if is_cursor { RESET } else { "" };
        let pointer = if is_cursor { "›" } else { " " };

        queue!(
            stdout,
            Print(format!(
                "  {bg} {pointer} {DIM}{connector}{RESET}{bg}{checkbox} {bg}{c}{icon}{RESET} {bg}{c}{}{end}{RESET}\r\n",
                item.change.path
            ))
        )?;
    }

    // Help line
    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}space{RESET} toggle  {DIM}a{RESET} all  {DIM}↑↓{RESET} move  {DIM}enter{RESET} confirm  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()
}

fn clear_display(item_count: usize, stdout: &mut io::Stdout) -> io::Result<()> {
    let total_lines = item_count + 4;
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
