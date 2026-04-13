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
    /// # or shorthand:
    /// feat = "Add a new feature"
    /// ```
    categories: Option<toml::value::Table>,
    /// Modules as a map:
    /// ```toml
    /// [modules]
    /// core = "Core logic"
    /// cli  = { desc = "CLI interface", path = "src/cli" }
    /// ```
    modules: Option<toml::value::Table>,
}

#[derive(Debug, Default, Deserialize)]
struct RawCommit {
    /// Commit message template: `{type}: {message}`
    template: Option<String>,
    /// Whether to prompt for a description body
    ask_description: Option<bool>,
    /// Whether to attach emoji before semantic prefix in the commit subject
    attach_emoji: Option<bool>,
    /// Backward compatibility for older config files
    show_emoji: Option<bool>,
    /// Automatically push after every commit without asking
    auto_push: Option<bool>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RawCategory {
    desc: Option<String>,
    emoji: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct RawModule {
    #[serde(alias = "description")]
    desc: Option<String>,
    #[serde(alias = "folder", alias = "folder_path")]
    path: Option<String>,
    #[serde(alias = "folders")]
    paths: Option<Vec<String>>,
}

// ── Resolved config ──────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Config {
    pub commit: CommitConfig,
    pub categories: Vec<Category>,
    pub modules: Vec<Module>,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: String,
    pub description: String,
    pub paths: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CommitConfig {
    /// Template for the commit subject. Placeholders: `{type}`, `{message}`
    pub template: String,
    /// Prompt the user for a long description
    pub ask_description: bool,
    /// Attach emoji before semantic prefix in the commit subject
    pub attach_emoji: bool,
    /// Push automatically after every commit (skip the "Push now?" prompt)
    pub auto_push: bool,
}

impl Default for CommitConfig {
    fn default() -> Self {
        Self {
            template: "$type($mod): $message".into(),
            ask_description: true,
            attach_emoji: false,
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
        }
    }
}

// ── Loading ──────────────────────────────────────────────

impl Config {
    /// Load config: global `~/sit.toml` merged with local `.sit/config.toml`.
    /// Backward-compatible fallbacks are `~/.sitrc` and local `.sitrc`.
    /// Local values override global; missing fields keep defaults.
    pub fn load() -> Self {
        let mut cfg = Config::default();

        // 1. Global
        if let Some(global_path) = global_config_path()
            && let Some(raw) = read_raw(&global_path)
        {
            cfg.apply(raw);
        }

        // 2. Local (walk up from cwd to find .sit/config.toml)
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
            if let Some(v) = c.attach_emoji.or(c.show_emoji) {
                self.commit.attach_emoji = v;
            }
            if let Some(v) = c.auto_push {
                self.commit.auto_push = v;
            }
        }

        // Categories – if provided, *replace* the full list
        if let Some(cats) = raw.categories {
            let mut used_shorthand = false;
            self.categories = cats
                .into_iter()
                .filter_map(|(name, value)| match value {
                    toml::Value::String(desc) => {
                        used_shorthand = true;
                        Some(Category {
                            name,
                            emoji: String::new(),
                            description: desc,
                        })
                    }
                    other => {
                        let rc: RawCategory = match other.try_into() {
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
                    }
                })
                .collect();

            if used_shorthand {
                self.commit.attach_emoji = false;
            }
        }

        // Modules – if provided, *replace* the full list
        if let Some(mods) = raw.modules {
            self.modules = mods
                .into_iter()
                .filter_map(|(name, value)| match value {
                    toml::Value::String(description) => Some(Module {
                        name,
                        description,
                        paths: vec![],
                    }),
                    other => {
                        let rm: RawModule = match other.try_into() {
                            Ok(v) => v,
                            Err(e) => {
                                crate::print::warn(&format!(
                                    "Invalid module entry for '{}': {}",
                                    name, e
                                ));
                                return None;
                            }
                        };

                        let mut paths = Vec::new();
                        if let Some(path) = rm.path {
                            let normalized = normalize_module_path(&path);
                            if !normalized.is_empty() {
                                paths.push(normalized);
                            }
                        }
                        if let Some(path_list) = rm.paths {
                            for path in path_list {
                                let normalized = normalize_module_path(&path);
                                if !normalized.is_empty() {
                                    paths.push(normalized);
                                }
                            }
                        }

                        Some(Module {
                            name,
                            description: rm.desc.unwrap_or_default(),
                            paths,
                        })
                    }
                })
                .collect();
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

        let subject = result.trim().to_owned();
        if !self.commit.attach_emoji {
            return subject;
        }

        let emoji = self
            .categories
            .iter()
            .find(|c| c.name == category)
            .map(|c| c.emoji.trim())
            .unwrap_or("");

        if emoji.is_empty() {
            subject
        } else {
            format!("{}{}", emoji, subject)
        }
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

    /// Recommend a module based on changed file path frequency.
    /// If multiple modules tie, the first one in config order wins.
    pub fn recommended_module_name<'a>(&'a self, files: &[String]) -> Option<&'a str> {
        let mut best_idx: Option<usize> = None;
        let mut best_score = 0usize;

        for (idx, module) in self.modules.iter().enumerate() {
            if module.paths.is_empty() {
                continue;
            }

            let mut score = 0usize;
            for file in files {
                for candidate in changed_file_candidates(file) {
                    if module
                        .paths
                        .iter()
                        .any(|path| file_matches_module_path(candidate, path))
                    {
                        score += 1;
                        break;
                    }
                }
            }

            if score > best_score {
                best_score = score;
                best_idx = Some(idx);
            }
        }

        best_idx
            .and_then(|idx| self.modules.get(idx))
            .map(|m| m.name.as_str())
    }

    /// Returns the default TOML content written by `sit init`.
    pub fn default_toml() -> &'static str {
        r#"[commit]
template        = "$type($mod): $message"
ask_description = true
auto_push       = false


[categories]
feat     = "Add a new feature"
fix      = "Fix a bug"
docs     = "Documentation changes"
style    = "Code style / formatting"
refactor = "Refactor code"
perf     = "Performance improvement"
test     = "Add or update tests"
build    = "Build system changes"
ci       = "CI/CD changes"
chore    = "Maintenance / chores"
revert   = "Revert a commit"
wip      = "Work in progress"
none     = "No category prefix"
"#
    }
}

// ── Helpers ──────────────────────────────────────────────

fn global_config_path() -> Option<PathBuf> {
    let home = dirs::home_dir()?;
    let primary = home.join("sit.toml");
    if primary.is_file() {
        return Some(primary);
    }

    let legacy = home.join(".sitrc");
    if legacy.is_file() {
        return Some(legacy);
    }

    None
}

fn local_config_path() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let primary = dir.join(".sit").join("config.toml");
        if primary.is_file() {
            return Some(primary);
        }

        let legacy = dir.join(".sitrc");
        if legacy.is_file() {
            return Some(legacy);
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

fn normalize_module_path(path: &str) -> String {
    path.trim()
        .trim_start_matches("./")
        .trim_end_matches('/')
        .to_owned()
}

fn changed_file_candidates(path: &str) -> impl Iterator<Item = &str> {
    path.split(" -> ").map(str::trim)
}

fn file_matches_module_path(file_path: &str, module_path: &str) -> bool {
    file_path == module_path || file_path.starts_with(&format!("{module_path}/"))
}
