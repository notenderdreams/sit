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
    /// Modules as a map:
    /// ```toml
    /// [modules]
    /// core = "Core logic"
    /// cli  = "CLI interface"
    /// ```
    modules: Option<toml::value::Table>,
    /// Clone settings
    clone: Option<RawClone>,
}

#[derive(Debug, Default, Deserialize)]
struct RawClone {
    /// Default directory for cloned repositories
    dir: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCommit {
    /// Commit message template: `{type}: {message}`
    template: Option<String>,
    /// Whether to prompt for a description body
    ask_description: Option<bool>,
    /// Whether to show emoji in the category picker
    show_emoji: Option<bool>,
    /// Automatically push after every commit without asking
    auto_push: Option<bool>,
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
    pub modules: Vec<Module>,
    pub clone: CloneConfig,
}

#[derive(Debug, Clone)]
pub struct CloneConfig {
    /// Default directory for clones (expandable: ~/projects, $HOME, etc.)
    pub dir: PathBuf,
}

impl Default for CloneConfig {
    fn default() -> Self {
        // Default: ~/projects
        let dir = dirs::home_dir()
            .map(|h| h.join("projects"))
            .unwrap_or_else(|| PathBuf::from("./projects"));
        Self { dir }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct CommitConfig {
    /// Template for the commit subject. Placeholders: `{type}`, `{message}`
    pub template: String,
    /// Prompt the user for a long description
    pub ask_description: bool,
    /// Show emoji column in the picker
    pub show_emoji: bool,
    /// Push automatically after every commit (skip the "Push now?" prompt)
    pub auto_push: bool,
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            template: "$type($mod): $message".into(),
            ask_description: true,
            show_emoji: true,
            auto_push: false,
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
            modules: vec![],
            clone: CloneConfig::default(),
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
        if let Some(global_path) = global_config_path()
            && let Some(raw) = read_raw(&global_path)
        {
            cfg.apply(raw);
        }

        // 2. Local (walk up from cwd to find .sitrc)
        if let Some(local_path) = local_config_path()
            && let Some(raw) = read_raw(&local_path)
        {
            cfg.apply(raw);
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
            if let Some(v) = c.auto_push {
                self.commit.auto_push = v;
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

        // Modules – if provided, *replace* the full list
        if let Some(mods) = raw.modules {
            self.modules = mods
                .into_iter()
                .filter_map(|(name, value)| {
                    let description = match value {
                        toml::Value::String(s) => s,
                        other => {
                            crate::print::warn(&format!(
                                "Invalid module entry for '{}': expected string, got {}",
                                name,
                                other.type_str()
                            ));
                            return None;
                        }
                    };
                    Some(Module { name, description })
                })
                .collect();
        }

        // Clone settings
        if let Some(c) = raw.clone
            && let Some(dir_str) = c.dir
        {
            // Expand ~ to home directory
            let expanded = if dir_str.starts_with('~') {
                if let Some(home) = dirs::home_dir() {
                    home.join(&dir_str[2..]) // skip ~/ or ~
                } else {
                    PathBuf::from(dir_str)
                }
            } else {
                PathBuf::from(dir_str)
            };
            self.clone.dir = expanded;
        }
    }

    /// Format a commit subject using the configured template.
    /// Placeholders: `$type`, `$mod`, `$message`
    pub fn format_subject(&self, category: &str, module: Option<&str>, message: &str) -> String {
        if category == "none" {
            return match module {
                Some(m) if !m.is_empty() => format!("({}): {}", m, message),
                _ => message.to_owned(),
            };
        }

        let mut result = self.commit.template.clone();
        result = result.replace("$type", category);
        result = result.replace("$message", message);

        // Handle $mod – if no module selected, collapse "($mod)" or just remove "$mod"
        match module {
            Some(m) if !m.is_empty() => {
                result = result.replace("$mod", m);
            }
            _ => {
                // Remove common patterns like "($mod)" or "[$mod]" or just "$mod"
                result = result.replace("($mod)", "");
                result = result.replace("[$mod]", "");
                result = result.replace("$mod", "");
                // Clean up double spaces or leading colons from collapsed module
                result = result.replace("  ", " ");
            }
        }

        result.trim().to_owned()
    }

    /// Build full commit message (subject + optional description).
    pub fn format_commit(
        &self,
        category: &str,
        module: Option<&str>,
        message: &str,
        description: &str,
    ) -> String {
        let subject = self.format_subject(category, module, message);
        if description.trim().is_empty() {
            subject
        } else {
            format!("{}\n\n{}", subject, description)
        }
    }

    /// Whether modules are configured.
    pub fn has_modules(&self) -> bool {
        !self.modules.is_empty()
    }

    /// Returns the default `.sitrc` TOML content written by `sit init`.
    pub fn default_toml() -> &'static str {
        r#"[commit]
template        = "$type($mod): $message"
ask_description = true
show_emoji      = true
auto_push       = false

[clone]
dir = "~/projects"

[categories]
feat     = { emoji = "✨",  desc = "Add a new feature" }
fix      = { emoji = "🐛",  desc = "Fix a bug" }
docs     = { emoji = "📚", desc = "Documentation changes" }
style    = { emoji = "🎨", desc = "Code style / formatting" }
refactor = { emoji = "♻️ ", desc = "Refactor code" }
perf     = { emoji = "⚡", desc = "Performance improvement" }
test     = { emoji = "🧪", desc = "Add or update tests" }
build    = { emoji = "📦", desc = "Build system changes" }
ci       = { emoji = "⚙️ ", desc = "CI/CD changes" }
chore    = { emoji = "🧹", desc = "Maintenance / chores" }
revert   = { emoji = "⏪", desc = "Revert a commit" }
wip      = { emoji = "🚧", desc = "Work in progress" }
none     = { emoji = "── ", desc = "No category prefix" }

# [modules]
# core = "Core logic"
# cli  = "CLI interface"
"#
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
