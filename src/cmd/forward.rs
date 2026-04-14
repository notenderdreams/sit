use crate::config::Config;
use crate::env_args::split_env_flags;
use crate::error::Result;
use crate::git;

use super::commit;

pub fn handle_external_or_category(
    cfg: &Config,
    args: &[String],
    hook_env: &[(String, String)],
) -> Result<()> {
    let Some(category) = args.first().map(|s| s.as_str()) else {
        return commit::interactive_commit(cfg, hook_env);
    };

    if !cfg.categories.iter().any(|c| c.name == category) {
        return git::forward_command(args);
    }

    let mut combined_env: Vec<(String, String)> = hook_env.to_vec();
    let (message_parts, extracted_env) = split_env_flags(&args[1..])?;
    combined_env.extend(extracted_env);

    let inline_message = if !message_parts.is_empty() {
        Some(message_parts.join(" "))
    } else {
        None
    };

    commit::commit_with_category_shortcut(cfg, category, inline_message, &combined_env)
}
