use std::io::{self, Write};

use crossterm::{
    cursor, execute, queue,
    terminal::{self, ClearType},
};

pub fn run_with_terminal<T, F>(f: F) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce(&mut io::Stdout) -> Result<T, Box<dyn std::error::Error>>,
{
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(stdout, cursor::Hide)?;

    let result = f(&mut stdout);

    execute!(stdout, cursor::Show)?;
    terminal::disable_raw_mode()?;

    result
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
