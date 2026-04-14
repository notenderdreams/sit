use crate::error::Result;
use crate::style::ICON_RENAMED;
use crate::{git, hooks, print};

pub fn push_branch(hook_env: &[(String, String)]) -> Result<()> {
    print::blank();
    do_push(hook_env)
}

pub(crate) fn do_push(hook_env: &[(String, String)]) -> Result<()> {
    let pre_push_env = hooks::merge_hook_env(&[], hook_env);
    let pre_push_env_refs = hooks::hook_env_refs(&pre_push_env);
    hooks::run_hook("pre-push", hooks::HookKind::Pre, &pre_push_env_refs)?;
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
    let post_push_env = hooks::merge_hook_env(
        &[
            ("SIT_REMOTE", &result.remote),
            ("SIT_BRANCH", &result.branch),
        ],
        hook_env,
    );
    let post_push_env_refs = hooks::hook_env_refs(&post_push_env);
    hooks::run_hook("post-push", hooks::HookKind::Post, &post_push_env_refs)?;
    print::blank();
    Ok(())
}

pub(crate) fn do_push_force(hook_env: &[(String, String)]) -> Result<()> {
    let pre_push_env = hooks::merge_hook_env(&[("SIT_FORCE", "1")], hook_env);
    let pre_push_env_refs = hooks::hook_env_refs(&pre_push_env);
    hooks::run_hook("pre-push", hooks::HookKind::Pre, &pre_push_env_refs)?;
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
    let post_push_env = hooks::merge_hook_env(
        &[
            ("SIT_REMOTE", &result.remote),
            ("SIT_BRANCH", &result.branch),
            ("SIT_FORCE", "1"),
        ],
        hook_env,
    );
    let post_push_env_refs = hooks::hook_env_refs(&post_push_env);
    hooks::run_hook("post-push", hooks::HookKind::Post, &post_push_env_refs)?;
    print::blank();
    Ok(())
}
