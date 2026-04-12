use colored::Colorize;

use crate::error::Result;
use crate::style::{DIM, RESET, TREE_LAST, TREE_MID};
use crate::{git, print, ui};

pub fn undo_commit() -> Result<()> {
    let last_message = git::last_commit_message()?;
    let last_files = git::last_commit_files()?;

    print::blank();
    print::header("Undo last commit?");
    print::blank();
    println!(
        "    {}",
        last_message.lines().next().unwrap_or(&last_message).bold()
    );
    print::blank();
    print::hint("Files (will remain staged):");
    let last = last_files.len().saturating_sub(1);
    for (i, f) in last_files.iter().enumerate() {
        let branch = if i == last { TREE_LAST } else { TREE_MID };
        println!("    {DIM}{branch}{RESET} {f}");
    }
    print::blank();

    if !ui::confirm_undo()? {
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    git::soft_reset()?;

    print::blank();
    print::success("Undone — changes are staged and ready to re-commit");
    print::blank();

    Ok(())
}
