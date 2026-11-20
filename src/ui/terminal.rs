use std::io::{self, Write};

use crossterm::{
    cursor, execute, queue,
    terminal::{self, ClearType},
};

use crate::error::Result;

/// Restores the terminal (cursor + cooked mode) on every exit path, including `?` and unwinding.
struct RawGuard;

impl Drop for RawGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), cursor::Show);
        let _ = terminal::disable_raw_mode();
    }
}

pub fn run_with_terminal<T, F>(f: F) -> Result<T>
where
    F: FnOnce(&mut io::Stdout) -> Result<T>,
{
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    let _guard = RawGuard;
    execute!(stdout, cursor::Hide)?;
    f(&mut stdout)
}

/// Clear exactly `n` rendered lines from the terminal.
pub fn clear_lines(n: usize, stdout: &mut io::Stdout) -> io::Result<()> {
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
