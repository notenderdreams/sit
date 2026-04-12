use crate::error::Result;
use crate::style::ICON_RENAMED;
use crate::{git, print, ui};

pub fn switch_branch() -> Result<()> {
    let branches = git::list_local_branches()?;
    let selected = ui::select_branch(&branches)?;

    if branches.iter().any(|b| b.name == selected) {
        git::switch_branch(&selected)?;
        print::blank();
        print::success_with_details("Switched", &format!("{ICON_RENAMED} {selected}"));
        print::blank();
        return Ok(());
    }

    if !ui::confirm_create_branch(&selected)? {
        print::blank();
        print::hint("Aborted");
        print::blank();
        return Ok(());
    }

    git::create_and_switch_branch(&selected)?;
    print::blank();
    print::success_with_details(
        "Created and switched",
        &format!("{ICON_RENAMED} {selected}"),
    );
    print::blank();
    Ok(())
}
