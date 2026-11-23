use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};

use crate::error::Result;
use crate::git::{FileChange, FileStatus};
use crate::style::{
    BG_SELECT, BOLD, CHECK_SELECTED, CHECK_UNSELECTED, CYAN, DIM, NAV_ARROWS, POINTER, RESET,
    TREE_LAST, TREE_MID,
};

const CHECK_PARTIAL: &str = "◐";
const FOLD_OPEN: &str = "▾";
const FOLD_CLOSED: &str = "▸";

struct Item {
    change: FileChange,
    selected: bool,
}

#[derive(Default)]
struct Dir {
    name: String,
    dirs: Vec<Dir>,
    files: Vec<(usize, String)>,
}

impl Dir {
    fn insert(&mut self, idx: usize, parts: &[&str]) {
        match parts {
            [] => {}
            [file] => self.files.push((idx, (*file).to_string())),
            [dir, rest @ ..] => {
                let pos = match self.dirs.iter().position(|d| d.name == *dir) {
                    Some(p) => p,
                    None => {
                        self.dirs.push(Dir {
                            name: (*dir).to_string(),
                            ..Dir::default()
                        });
                        self.dirs.len() - 1
                    }
                };
                self.dirs[pos].insert(idx, rest);
            }
        }
    }
}

enum Kind {
    Dir { expanded: bool },
    File(usize),
}

/// Pre-order flattening of the tree: a dir's descendants follow it contiguously.
struct Entry {
    name: String,
    depth: usize,
    parent: Option<usize>,
    last: bool,
    kind: Kind,
}

fn flatten(dir: &Dir, depth: usize, parent: Option<usize>, out: &mut Vec<Entry>) {
    let total = dir.dirs.len() + dir.files.len();
    let mut n = 0;
    for d in &dir.dirs {
        n += 1;
        out.push(Entry {
            name: d.name.clone(),
            depth,
            parent,
            last: n == total,
            kind: Kind::Dir { expanded: true },
        });
        let me = out.len() - 1;
        flatten(d, depth + 1, Some(me), out);
    }
    for (idx, name) in &dir.files {
        n += 1;
        out.push(Entry {
            name: name.clone(),
            depth,
            parent,
            last: n == total,
            kind: Kind::File(*idx),
        });
    }
}

fn build_entries(items: &[Item]) -> Vec<Entry> {
    let mut root = Dir::default();
    for (i, item) in items.iter().enumerate() {
        let parts: Vec<&str> = item.change.path.split('/').collect();
        root.insert(i, &parts);
    }
    sort_dirs(&mut root);
    let mut out = Vec::new();
    flatten(&root, 0, None, &mut out);
    out
}

fn sort_dirs(dir: &mut Dir) {
    dir.dirs.sort_by(|a, b| a.name.cmp(&b.name));
    dir.dirs.iter_mut().for_each(sort_dirs);
}

/// Entry indices currently shown (descendants of collapsed dirs are hidden).
fn visible(entries: &[Entry]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < entries.len() {
        out.push(i);
        let depth = entries[i].depth;
        let collapsed = matches!(entries[i].kind, Kind::Dir { expanded: false });
        i += 1;
        if collapsed {
            while i < entries.len() && entries[i].depth > depth {
                i += 1;
            }
        }
    }
    out
}

/// Item indices of every file under `entries[i]` (itself if a file).
fn files_under(entries: &[Entry], i: usize) -> Vec<usize> {
    let depth = entries[i].depth;
    let mut out = Vec::new();
    if let Kind::File(idx) = entries[i].kind {
        return vec![idx];
    }
    for e in &entries[i + 1..] {
        if e.depth <= depth {
            break;
        }
        if let Kind::File(idx) = e.kind {
            out.push(idx);
        }
    }
    out
}

pub fn pick_files(changes: Vec<FileChange>) -> Result<Vec<String>> {
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

    let entries = build_entries(&items);
    let mut cursor_pos: usize = 0;
    let mut stdout = io::stdout();

    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = run_loop(&mut items, entries, &mut cursor_pos, &mut stdout);

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
    mut entries: Vec<Entry>,
    cursor_pos: &mut usize,
    stdout: &mut io::Stdout,
) -> io::Result<bool> {
    let mut last_height = render(items, &entries, *cursor_pos, 0, stdout)?;

    loop {
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let vis = visible(&entries);
                    let pos = vis.iter().position(|&e| e == *cursor_pos).unwrap_or(0);
                    if pos > 0 {
                        *cursor_pos = vis[pos - 1];
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    let vis = visible(&entries);
                    let pos = vis.iter().position(|&e| e == *cursor_pos).unwrap_or(0);
                    if pos + 1 < vis.len() {
                        *cursor_pos = vis[pos + 1];
                    }
                }
                KeyCode::Left | KeyCode::Char('h') => {
                    let cur = *cursor_pos;
                    match &mut entries[cur].kind {
                        Kind::Dir { expanded } if *expanded => *expanded = false,
                        _ => {
                            if let Some(p) = entries[cur].parent {
                                *cursor_pos = p;
                            }
                        }
                    }
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    if let Kind::Dir { expanded } = &mut entries[*cursor_pos].kind {
                        if *expanded {
                            *cursor_pos += 1;
                        } else {
                            *expanded = true;
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    let idxs = files_under(&entries, *cursor_pos);
                    let all_on = idxs.iter().all(|&i| items[i].selected);
                    for i in idxs {
                        items[i].selected = !all_on;
                    }
                }
                KeyCode::Char('a') => {
                    let all_on = items.iter().all(|i| i.selected);
                    for item in items.iter_mut() {
                        item.selected = !all_on;
                    }
                }
                KeyCode::Enter => {
                    clear_display(last_height, stdout)?;
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
                        let conn = if i == count - 1 { TREE_LAST } else { TREE_MID };
                        queue!(
                            stdout,
                            Print(format!(
                                "    {DIM}{conn} {RESET}{c}{icon} {}{RESET}\r\n",
                                item.change.path
                            ))
                        )?;
                    }
                    queue!(stdout, Print("\r\n"))?;
                    stdout.flush()?;
                    return Ok(true);
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    clear_display(last_height, stdout)?;
                    return Ok(false);
                }
                _ => {}
            }
            last_height = render(items, &entries, *cursor_pos, last_height, stdout)?;
        }
    }
}

fn render(
    items: &[Item],
    entries: &[Entry],
    cursor_pos: usize,
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

    let selected_count = items.iter().filter(|i| i.selected).count();

    // Header
    queue!(
        stdout,
        Print(format!(
            "\r\n  {BOLD}  Changes{RESET} {DIM}({selected_count}/{} selected){RESET}\r\n\r\n",
            items.len()
        ))
    )?;

    let vis = visible(entries);
    for &ei in &vis {
        let e = &entries[ei];
        let bg = if ei == cursor_pos { BG_SELECT } else { "" };
        let end = if ei == cursor_pos { RESET } else { "" };
        let pointer = if ei == cursor_pos { POINTER } else { " " };

        // guide columns for ancestors, then this entry's connector
        let mut guides = Vec::new();
        let mut p = e.parent;
        while let Some(pi) = p {
            guides.push(if entries[pi].last { "    " } else { "│   " });
            p = entries[pi].parent;
        }
        guides.reverse();
        let prefix = guides.concat();
        let connector = if e.last { TREE_LAST } else { TREE_MID };

        let body = match e.kind {
            Kind::File(idx) => {
                let item = &items[idx];
                let c = item.change.status.color_code();
                let icon = item.change.status.icon();
                let checkbox = if item.selected {
                    format!("{c}{CHECK_SELECTED}{RESET}")
                } else {
                    format!("{DIM}{CHECK_UNSELECTED}{RESET}")
                };
                format!("{checkbox} {bg}{c}{icon}{RESET} {bg}{c}{}", e.name)
            }
            Kind::Dir { expanded } => {
                let idxs = files_under(entries, ei);
                let on = idxs.iter().filter(|&&i| items[i].selected).count();
                let checkbox = if on == idxs.len() {
                    format!("{CYAN}{CHECK_SELECTED}{RESET}")
                } else if on == 0 {
                    format!("{DIM}{CHECK_UNSELECTED}{RESET}")
                } else {
                    format!("{CYAN}{CHECK_PARTIAL}{RESET}")
                };
                let fold = if expanded { FOLD_OPEN } else { FOLD_CLOSED };
                format!("{checkbox} {bg}{DIM}{fold}{RESET}{bg} {BOLD}{}/", e.name)
            }
        };

        queue!(
            stdout,
            Print(format!(
                "  {bg} {pointer} {DIM}{prefix}{connector}{RESET}{bg}{body}{end}{RESET}\r\n"
            ))
        )?;
    }

    // Help line
    queue!(
        stdout,
        Print(format!(
            "\r\n  {DIM}space{RESET} toggle  {DIM}a{RESET} all  {DIM}{NAV_ARROWS}{RESET} move  {DIM}←→{RESET} fold  {DIM}enter{RESET} confirm  {DIM}esc{RESET} cancel"
        ))
    )?;

    stdout.flush()?;

    // leading \r\n(1) + header \r\n(1) + blank \r\n(1) + rows + help \r\n(1) = rows + 4
    Ok(vis.len() + 4)
}

fn clear_display(height: usize, stdout: &mut io::Stdout) -> io::Result<()> {
    if height == 0 {
        return Ok(());
    }
    queue!(stdout, cursor::MoveToColumn(0))?;
    for _ in 0..height {
        queue!(
            stdout,
            terminal::Clear(ClearType::CurrentLine),
            cursor::MoveUp(1)
        )?;
    }
    queue!(stdout, terminal::Clear(ClearType::CurrentLine))?;
    stdout.flush()
}
