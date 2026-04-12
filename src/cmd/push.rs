use crate::error::Result;
use crate::style::ICON_RENAMED;
use crate::{git, hooks, print};

pub fn push_branch() -> Result<()> {
    print::blank();
    do_push()
}

pub(crate) fn do_push() -> Result<()> {
    hooks::run_hook("pre-push", hooks::HookKind::Pre, &[])?;
    let result = git::push()?;
    if result.set_upstream {
        print::success_with_details(
            "Pushed",
            &format!(
                "{ICON_RENAMED} {}/{} (upstream set)",
                result.remote, result.branch
            ),
        );
    } else {
        print::success_with_details(
            "Pushed",
            &format!("{ICON_RENAMED} {}/{}", result.remote, result.branch),
        );
    }
    hooks::run_hook(
        "post-push",
        hooks::HookKind::Post,
        &[
            ("SIT_REMOTE", &result.remote),
            ("SIT_BRANCH", &result.branch),
        ],
    )?;
    print::blank();
    Ok(())
}

pub(crate) fn do_push_force() -> Result<()> {
    hooks::run_hook("pre-push", hooks::HookKind::Pre, &[("SIT_FORCE", "1")])?;
    let result = git::push_force()?;
    if result.set_upstream {
        print::success_with_details(
            "Pushed",
            &format!(
                "{ICON_RENAMED} {}/{} (upstream set)",
                result.remote, result.branch
            ),
        );
    } else {
        print::success_with_details(
            "Pushed",
            &format!(
                "{ICON_RENAMED} {}/{} (force-with-lease)",
                result.remote, result.branch
            ),
        );
    }
    hooks::run_hook(
        "post-push",
        hooks::HookKind::Post,
        &[
            ("SIT_REMOTE", &result.remote),
            ("SIT_BRANCH", &result.branch),
            ("SIT_FORCE", "1"),
        ],
    )?;
    print::blank();
    Ok(())
}
