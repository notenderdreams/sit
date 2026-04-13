use crate::error::Result;
use crate::style::ICON_RENAMED;
use crate::{git, print};

pub fn connect_repo(username: &str, repo: &str) -> Result<()> {
    print::blank();
    print::header(&format!("Connecting to github.com/{username}/{repo}"));
    print::blank();

    git::remote_add_origin(username, repo)?;
    print::success_with_details(
        "Remote added",
        &format!("origin {ICON_RENAMED} https://github.com/{username}/{repo}.git"),
    );

    git::branch_rename_to_main()?;
    print::success_with_details("Branch", "renamed to main");

    git::push_origin_main()?;
    print::success_with_details(
        "Pushed",
        &format!("{ICON_RENAMED} origin/main (upstream set)"),
    );

    print::blank();
    Ok(())
}
