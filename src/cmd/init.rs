use std::path::Path;

use crate::config::Config;
use crate::error::Result;
use crate::{hooks, print};

pub fn init_config() -> Result<()> {
    let sit_dir = Path::new(".sit");
    let hooks_dir = sit_dir.join("hooks");
    let config_path = sit_dir.join("config.toml");
    let legacy_path = Path::new(".sitrc");

    if config_path.exists() {
        return Err(".sit/config.toml already exists in the current directory".into());
    }

    std::fs::create_dir_all(&hooks_dir)?;

    let migrated_legacy = if legacy_path.exists() {
        match std::fs::rename(legacy_path, &config_path) {
            Ok(()) => true,
            Err(_) => {
                std::fs::copy(legacy_path, &config_path)?;
                std::fs::remove_file(legacy_path)?;
                true
            }
        }
    } else {
        std::fs::write(&config_path, Config::default_toml())?;
        false
    };

    write_hook_template(&hooks_dir.join("pre-commit"), hooks::PRE_COMMIT_TEMPLATE)?;
    write_hook_template(&hooks_dir.join("post-commit"), hooks::POST_COMMIT_TEMPLATE)?;
    write_hook_template(&hooks_dir.join("pre-push"), hooks::PRE_PUSH_TEMPLATE)?;
    write_hook_template(&hooks_dir.join("post-push"), hooks::POST_PUSH_TEMPLATE)?;

    print::blank();
    if migrated_legacy {
        print::success("Migrated .sitrc to .sit/config.toml and created hook templates");
    } else {
        print::success("Created .sit/config.toml and hook templates");
    }
    print::hint("Edit .sit/config.toml and uncomment hook examples in .sit/hooks as needed.");
    print::blank();

    Ok(())
}

fn write_hook_template(path: &std::path::Path, template: &str) -> Result<()> {
    if !path.exists() {
        std::fs::write(path, template)?;
    }
    Ok(())
}
