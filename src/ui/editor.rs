//! Line/multi-line text editor with readline-style keys.
//!
//! `Buffer` is pure (testable) editing state; `edit` drives it from the terminal.

use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{
        self, DisableBracketedPaste, EnableBracketedPaste, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers,
    },
    execute, queue,
    style::Print,
    terminal::{self, ClearType},
};
use unicode_width::UnicodeWidthChar;

use crate::error::Result;
use crate::style::{DIM, RESET};

// ───────────────────────── buffer ─────────────────────────

#[derive(Default, Clone)]
pub struct Buffer {
    chars: Vec<char>,
    cur: usize,
    /// Column to return to when moving vertically through shorter lines.
    goal_col: Option<usize>,
}

fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn normalize(s: &str) -> Vec<char> {
    s.replace("\r\n", "\n")
        .replace('\r', "\n")
        .chars()
        .collect()
}

impl Buffer {
    pub fn new(text: &str) -> Self {
        let chars = normalize(text);
        Self {
            cur: chars.len(),
            chars,
            goal_col: None,
        }
    }

    pub fn text(&self) -> String {
        self.chars.iter().collect()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    #[cfg(test)]
    pub fn cursor(&self) -> usize {
        self.cur
    }

    fn line_start(&self, pos: usize) -> usize {
        self.chars[..pos]
            .iter()
            .rposition(|&c| c == '\n')
            .map_or(0, |i| i + 1)
    }

    fn line_end(&self, pos: usize) -> usize {
        self.chars[pos..]
            .iter()
            .position(|&c| c == '\n')
            .map_or(self.chars.len(), |i| pos + i)
    }

    /// (line index, column in chars) of the cursor.
    pub fn line_col(&self) -> (usize, usize) {
        let line = self.chars[..self.cur]
            .iter()
            .filter(|&&c| c == '\n')
            .count();
        (line, self.cur - self.line_start(self.cur))
    }

    fn touch(&mut self) {
        self.goal_col = None;
    }

    pub fn insert_str(&mut self, s: &str) {
        self.touch();
        let new = normalize(s);
        let n = new.len();
        self.chars.splice(self.cur..self.cur, new);
        self.cur += n;
    }

    pub fn insert_char(&mut self, c: char) {
        self.touch();
        self.chars.insert(self.cur, c);
        self.cur += 1;
    }

    pub fn backspace(&mut self) {
        self.touch();
        if self.cur > 0 {
            self.cur -= 1;
            self.chars.remove(self.cur);
        }
    }

    pub fn delete(&mut self) {
        self.touch();
        if self.cur < self.chars.len() {
            self.chars.remove(self.cur);
        }
    }

    pub fn left(&mut self) {
        self.touch();
        self.cur = self.cur.saturating_sub(1);
    }

    pub fn right(&mut self) {
        self.touch();
        self.cur = (self.cur + 1).min(self.chars.len());
    }

    pub fn home(&mut self) {
        self.touch();
        self.cur = self.line_start(self.cur);
    }

    pub fn end(&mut self) {
        self.touch();
        self.cur = self.line_end(self.cur);
    }

    pub fn buffer_start(&mut self) {
        self.touch();
        self.cur = 0;
    }

    pub fn buffer_end(&mut self) {
        self.touch();
        self.cur = self.chars.len();
    }

    pub fn up(&mut self) {
        let start = self.line_start(self.cur);
        if start == 0 {
            return;
        }
        let col = self.goal_col.unwrap_or(self.cur - start);
        let prev_start = self.line_start(start - 1);
        self.cur = prev_start + col.min(start - 1 - prev_start);
        self.goal_col = Some(col);
    }

    pub fn down(&mut self) {
        let end = self.line_end(self.cur);
        if end >= self.chars.len() {
            return;
        }
        let col = self
            .goal_col
            .unwrap_or(self.cur - self.line_start(self.cur));
        let next_start = end + 1;
        self.cur = next_start + col.min(self.line_end(next_start) - next_start);
        self.goal_col = Some(col);
    }

    fn word_left_pos(&self) -> usize {
        let mut i = self.cur;
        while i > 0 && !is_word(self.chars[i - 1]) {
            i -= 1;
        }
        while i > 0 && is_word(self.chars[i - 1]) {
            i -= 1;
        }
        i
    }

    fn word_right_pos(&self) -> usize {
        let n = self.chars.len();
        let mut i = self.cur;
        while i < n && !is_word(self.chars[i]) {
            i += 1;
        }
        while i < n && is_word(self.chars[i]) {
            i += 1;
        }
        i
    }

    pub fn word_left(&mut self) {
        self.touch();
        self.cur = self.word_left_pos();
    }

    pub fn word_right(&mut self) {
        self.touch();
        self.cur = self.word_right_pos();
    }

    pub fn delete_word_back(&mut self) {
        self.touch();
        let from = self.word_left_pos();
        self.chars.drain(from..self.cur);
        self.cur = from;
    }

    pub fn delete_word_forward(&mut self) {
        self.touch();
        let to = self.word_right_pos();
        self.chars.drain(self.cur..to);
    }

    /// Ctrl+K: delete to end of line (or the newline itself at end of line).
    pub fn kill_to_end(&mut self) {
        self.touch();
        let end = self.line_end(self.cur);
        let to = if end == self.cur {
            (end + 1).min(self.chars.len())
        } else {
            end
        };
        self.chars.drain(self.cur..to);
    }

    /// Ctrl+U: delete to start of line.
    pub fn kill_to_start(&mut self) {
        self.touch();
        let start = self.line_start(self.cur);
        self.chars.drain(start..self.cur);
        self.cur = start;
    }

    /// Enter after a trailing `\` turns the backslash into a newline.
    pub fn continue_line(&mut self) -> bool {
        if self.cur > 0 && self.chars[self.cur - 1] == '\\' {
            self.chars[self.cur - 1] = '\n';
            self.touch();
            true
        } else {
            false
        }
    }
}

// ───────────────────────── rendering ─────────────────────────

pub struct EditorOptions<'a> {
    /// Styled prompt shown before the first line.
    pub prefix: &'a str,
    /// Visible width of `prefix` (continuation lines are indented by this much).
    pub prefix_width: usize,
    pub initial: &'a str,
    pub placeholder: &'a str,
    /// Enter submits; newlines are entered via Alt+Enter, Ctrl+J or `\`+Enter.
    pub multiline: bool,
    /// Esc returns an empty string instead of cancelling.
    pub esc_skips: bool,
}

fn text_width(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

fn term_width() -> usize {
    terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
        .max(8)
}

/// Rows a line occupies. A line exactly filling the width takes one extra row
/// (we emit an explicit wrap so cursor math never depends on pending-wrap state).
fn rows_for(width: usize, term_w: usize) -> usize {
    width / term_w + 1
}

struct Frame {
    /// Rows from the top of the block to the cursor.
    cursor_row: usize,
}

fn render(
    out: &mut impl Write,
    opts: &EditorOptions,
    buf: &Buffer,
    prev: Option<&Frame>,
    hint: Option<&str>,
) -> io::Result<Frame> {
    let term_w = term_width();
    let text = buf.text();
    let pad = " ".repeat(opts.prefix_width);

    if let Some(prev) = prev
        && prev.cursor_row > 0
    {
        queue!(out, cursor::MoveUp(prev.cursor_row as u16))?;
    }
    queue!(
        out,
        cursor::MoveToColumn(0),
        terminal::Clear(ClearType::FromCursorDown)
    )?;

    let (cur_line, cur_col) = buf.line_col();
    let (mut rows, mut cursor_row, mut cursor_col) = (0usize, 0usize, 0usize);

    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            queue!(out, Print("\r\n"))?;
        }
        queue!(out, Print(if i == 0 { opts.prefix } else { &pad }))?;
        if text.is_empty() {
            queue!(out, Print(format!("{DIM}{}{RESET}", opts.placeholder)))?;
        } else {
            queue!(out, Print(line))?;
        }
        let shown = if text.is_empty() {
            text_width(opts.placeholder)
        } else {
            text_width(line)
        };
        let w = opts.prefix_width + shown;
        if w.is_multiple_of(term_w) {
            queue!(out, Print(" "), cursor::MoveToColumn(0))?;
        }
        if i == cur_line {
            let before =
                opts.prefix_width + text_width(&line.chars().take(cur_col).collect::<String>());
            cursor_row = rows + before / term_w;
            cursor_col = before % term_w;
        }
        rows += rows_for(w, term_w);
    }

    let mut last_row = rows - 1;
    if let Some(h) = hint {
        let h: String = h
            .chars()
            .take(term_w.saturating_sub(opts.prefix_width + 1))
            .collect();
        queue!(out, Print("\r\n"), Print(format!("{DIM}{pad}{h}{RESET}")))?;
        last_row += 1;
    }

    // Cursor sits on the last printed row; move it to its logical position.
    let up = last_row.saturating_sub(cursor_row);
    if up > 0 {
        queue!(out, cursor::MoveUp(up as u16))?;
    }
    queue!(out, cursor::MoveToColumn(cursor_col as u16))?;
    out.flush()?;
    Ok(Frame { cursor_row })
}

/// Redraw without the hint, then move below the block so later output starts on a fresh line.
fn finish(
    out: &mut impl Write,
    opts: &EditorOptions,
    buf: &Buffer,
    frame: &Frame,
) -> io::Result<()> {
    let frame = render(out, opts, buf, Some(frame), None)?;
    let term_w = term_width();
    let text = buf.text();
    let total: usize = text
        .split('\n')
        .map(|l| rows_for(opts.prefix_width + text_width(l), term_w))
        .sum();
    let down = total.saturating_sub(1).saturating_sub(frame.cursor_row);
    if down > 0 {
        queue!(out, cursor::MoveDown(down as u16))?;
    }
    queue!(out, Print("\r\n"))?;
    out.flush()
}

/// Run the editor. Returns the edited text (untrimmed).
pub fn edit(opts: &EditorOptions) -> Result<String> {
    super::terminal::run_with_terminal(|stdout| {
        execute!(stdout, cursor::Show)?;
        let _ = execute!(stdout, EnableBracketedPaste);
        let result = edit_loop(stdout, opts);
        let _ = execute!(stdout, DisableBracketedPaste);
        result
    })
}

fn edit_loop(stdout: &mut io::Stdout, opts: &EditorOptions) -> Result<String> {
    let mut buf = if opts.multiline {
        Buffer::new(opts.initial)
    } else {
        Buffer::new(&opts.initial.replace('\n', " "))
    };
    let hint = opts
        .multiline
        .then_some("enter submit · alt+enter / ctrl+j / \\+enter newline · esc skip");
    let mut frame = render(stdout, opts, &buf, None, hint)?;

    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                kind: KeyEventKind::Release,
                ..
            }) => continue,
            Event::Key(key) => {
                let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                let alt = key.modifiers.contains(KeyModifiers::ALT);
                let shift = key.modifiers.contains(KeyModifiers::SHIFT);
                match key.code {
                    KeyCode::Enter => {
                        if opts.multiline && (alt || shift) {
                            buf.insert_char('\n');
                        } else if !(opts.multiline && buf.continue_line()) {
                            finish(stdout, opts, &buf, &frame)?;
                            return Ok(buf.text());
                        }
                    }
                    KeyCode::Esc => {
                        finish(stdout, opts, &buf, &frame)?;
                        return if opts.esc_skips {
                            Ok(String::new())
                        } else {
                            Err("Cancelled by user".into())
                        };
                    }
                    KeyCode::Char('c') if ctrl => {
                        finish(stdout, opts, &buf, &frame)?;
                        return Err("Cancelled by user".into());
                    }
                    KeyCode::Char('j') if ctrl && opts.multiline => buf.insert_char('\n'),
                    KeyCode::Char('a') if ctrl => buf.home(),
                    KeyCode::Char('e') if ctrl => buf.end(),
                    KeyCode::Char('b') if ctrl => buf.left(),
                    KeyCode::Char('f') if ctrl => buf.right(),
                    KeyCode::Char('p') if ctrl => buf.up(),
                    KeyCode::Char('n') if ctrl => buf.down(),
                    KeyCode::Char('d') if ctrl => buf.delete(),
                    KeyCode::Char('k') if ctrl => buf.kill_to_end(),
                    KeyCode::Char('u') if ctrl => buf.kill_to_start(),
                    // Many terminals send Ctrl+Backspace as 0x08 (Ctrl+H).
                    KeyCode::Char('w' | 'h') if ctrl => buf.delete_word_back(),
                    KeyCode::Char('b') if alt => buf.word_left(),
                    KeyCode::Char('f') if alt => buf.word_right(),
                    KeyCode::Char('d') if alt => buf.delete_word_forward(),
                    KeyCode::Char(c) if !ctrl && !alt => buf.insert_char(c),
                    KeyCode::Backspace if ctrl || alt => buf.delete_word_back(),
                    KeyCode::Backspace => buf.backspace(),
                    KeyCode::Delete if ctrl || alt => buf.delete_word_forward(),
                    KeyCode::Delete => buf.delete(),
                    KeyCode::Left if ctrl || alt => buf.word_left(),
                    KeyCode::Right if ctrl || alt => buf.word_right(),
                    KeyCode::Left => buf.left(),
                    KeyCode::Right => buf.right(),
                    KeyCode::Up => buf.up(),
                    KeyCode::Down => buf.down(),
                    KeyCode::Home if ctrl => buf.buffer_start(),
                    KeyCode::End if ctrl => buf.buffer_end(),
                    KeyCode::Home => buf.home(),
                    KeyCode::End => buf.end(),
                    _ => continue,
                }
            }
            Event::Paste(text) => {
                if opts.multiline {
                    buf.insert_str(&text);
                } else {
                    buf.insert_str(&text.replace("\r\n", " ").replace(['\n', '\r'], " "));
                }
            }
            Event::Resize(..) => {}
            _ => continue,
        }
        frame = render(stdout, opts, &buf, Some(&frame), hint)?;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> Buffer {
        Buffer::new(s)
    }

    #[test]
    fn up_down_keep_goal_column() {
        let mut x = b("hello\nhi\nworld");
        x.cur = 4;
        x.down();
        assert_eq!(x.line_col(), (1, 2));
        x.down();
        assert_eq!(x.line_col(), (2, 4));
        x.up();
        x.up();
        assert_eq!(x.line_col(), (0, 4));
        x.up();
        assert_eq!(x.line_col(), (0, 4));
    }

    #[test]
    fn word_motion_and_delete() {
        let mut x = b("foo bar_baz  qux");
        x.word_left();
        assert_eq!(x.cursor(), 13);
        x.word_left();
        assert_eq!(x.cursor(), 4);
        x.word_right();
        assert_eq!(x.cursor(), 11);
        x.delete_word_back();
        assert_eq!(x.text(), "foo   qux");
    }

    #[test]
    fn word_delete_forward() {
        let mut x = b("foo bar");
        x.buffer_start();
        x.delete_word_forward();
        assert_eq!(x.text(), " bar");
    }

    #[test]
    fn home_end_are_per_line() {
        let mut x = b("ab\ncd");
        x.home();
        assert_eq!(x.cursor(), 3);
        x.up();
        x.end();
        assert_eq!(x.cursor(), 2);
    }

    #[test]
    fn kill_line() {
        let mut x = b("ab\ncd");
        x.cur = 1;
        x.kill_to_end();
        assert_eq!(x.text(), "a\ncd");
        x.kill_to_end();
        assert_eq!(x.text(), "acd");
        x.kill_to_start();
        assert_eq!(x.text(), "cd");
    }

    #[test]
    fn backslash_continuation_and_unicode() {
        let mut x = b("héllo\\");
        assert!(x.continue_line());
        assert_eq!(x.text(), "héllo\n");
        x.insert_str("wörld\r\nx");
        assert_eq!(x.text(), "héllo\nwörld\nx");
        assert!(!b("plain").continue_line());
    }

    #[test]
    fn edits_at_boundaries_do_not_panic() {
        let mut x = b("");
        x.backspace();
        x.delete();
        x.left();
        x.up();
        x.down();
        x.delete_word_back();
        x.kill_to_end();
        assert!(x.is_empty());
    }
}
