use crate::config::Config;
use crate::error::Result;
use crate::git;

use super::commit;

pub fn handle_external_or_category(cfg: &Config, args: &[String]) -> Result<()> {
    let Some(category) = args.first().map(|s| s.as_str()) else {
        return commit::interactive_commit(cfg);
    };

    if !cfg.categories.iter().any(|c| c.name == category) {
        return git::forward_command(args);
    }

    let inline_message = if args.len() > 1 {
        Some(args[1..].join(" "))
    } else {
        None
    };

    commit::commit_with_category_shortcut(cfg, category, inline_message)
}
