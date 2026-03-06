use std::path::{Path, PathBuf};
use std::{env, fs};

use serde::Deserialize;

use crate::categories::{Category, DEFAULT_CATEGORIES};

// ── Raw TOML shapes ──────────────────────────────────────

#[derive(Debug, Default, Deserialize)]
struct RawConfig {
    commit: Option<RawCommit>,
    /// Categories as a map:
    /// ```toml
    /// [categories]
    /// feat = { emoji = "✨", desc = "Add a new feature" }
    /// fix  = { emoji = "🐛", desc = "Fix a bug" }
    /// ```
    categories: Option<toml::value::Table>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCommit {
    /// Commit message template: `{type}: {message}`
    template: Option<String>,
    /// Whether to prompt for a description body
    ask_description: Option<bool>,
    /// Whether to show emoji in the category picker
    show_emoji: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RawCategory {
    desc: Option<String>,
    emoji: Option<String>,
}

// ── Resolved config ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Config {
    pub commit: CommitConfig,
    pub categories: Vec<Category>,
}

#[derive(Debug, Clone)]
pub struct CommitConfig {
    /// Template for the commit subject. Placeholders: `{type}`, `{message}`
    pub template: String,
    /// Prompt the user for a long description
    pub ask_description: bool,
    /// Show emoji column in the picker
    pub show_emoji: bool,
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            template: "{type}: {message}".into(),
            ask_description: true,
            show_emoji: true,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            commit: CommitConfig::default(),
            categories: DEFAULT_CATEGORIES
                .iter()
                .map(|c| Category {
                    name: c.name.to_owned(),
                    description: c.description.to_owned(),
                    emoji: c.emoji.to_owned(),
                })
                .collect(),
        }
    }
}

// ── Loading ──────────────────────────────────────────────

impl Config {
    /// Load config: global `~/.sitrc` merged with local `.sitrc`.
    /// Local values override global; missing fields keep defaults.
    pub fn load() -> Self {
        let mut cfg = Config::default();

        // 1. Global
        if let Some(global_path) = global_config_path() {
            if let Some(raw) = read_raw(&global_path) {
                cfg.apply(raw);
            }
        }

        // 2. Local (walk up from cwd to find .sitrc)
        if let Some(local_path) = local_config_path() {
            if let Some(raw) = read_raw(&local_path) {
                cfg.apply(raw);
            }
        }

        cfg
    }

    /// Merge a parsed raw config on top of self.
    fn apply(&mut self, raw: RawConfig) {
        // Commit settings
        if let Some(c) = raw.commit {
            if let Some(t) = c.template {
                self.commit.template = t;
            }
            if let Some(v) = c.ask_description {
                self.commit.ask_description = v;
            }
            if let Some(v) = c.show_emoji {
                self.commit.show_emoji = v;
            }
        }

        // Categories – if provided, *replace* the full list
        if let Some(cats) = raw.categories {
            self.categories = cats
                .into_iter()
                .filter_map(|(name, value)| {
                    let rc: RawCategory = match value.try_into() {
                        Ok(v) => v,
                        Err(e) => {
                            crate::print::warn(&format!(
                                "Invalid category entry for '{}': {}",
                                name, e
                            ));
                            return None;
                        }
                    };

                    Some(Category {
                        name,
                        emoji: rc.emoji.unwrap_or_default(),
                        description: rc.desc.unwrap_or_default(),
                    })
                })
                .collect();
        }
    }

    /// Format a commit subject using the configured template.
    pub fn format_subject(&self, category: &str, message: &str) -> String {
        if category == "none" {
            return message.to_owned();
        }
        self.commit
            .template
            .replace("{type}", category)
            .replace("{message}", message)
    }

    /// Build full commit message (subject + optional description).
    pub fn format_commit(&self, category: &str, message: &str, description: &str) -> String {
        let subject = self.format_subject(category, message);
        if description.trim().is_empty() {
            subject
        } else {
            format!("{}\n\n{}", subject, description)
        }
    }
}

// ── Helpers ──────────────────────────────────────────────

fn global_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".sitrc"))
}

fn local_config_path() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join(".sitrc");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            break;
        }
    }
    None
}

fn read_raw(path: &Path) -> Option<RawConfig> {
    let content = fs::read_to_string(path).ok()?;
    match toml::from_str::<RawConfig>(&content) {
        Ok(raw) => Some(raw),
        Err(e) => {
            crate::print::warn(&format!("Failed to parse {}: {}", path.display(), e));
            None
        }
    }
}
