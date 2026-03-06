#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
    pub description: String,
    pub emoji: String,
}

/// Compile-time category definition (for the built-in defaults).
pub struct StaticCategory {
    pub name: &'static str,
    pub description: &'static str,
    pub emoji: &'static str,
}

/// Built-in default categories (used when no config overrides them).
pub const DEFAULT_CATEGORIES: &[StaticCategory] = &[
    StaticCategory {
        name: "feat",
        description: "Add a new feature",
        emoji: "✨",
    },
    StaticCategory {
        name: "fix",
        description: "Fix a bug",
        emoji: "🐛",
    },
    StaticCategory {
        name: "docs",
        description: "Documentation changes",
        emoji: "📚",
    },
    StaticCategory {
        name: "style",
        description: "Code style / formatting",
        emoji: "🎨",
    },
    StaticCategory {
        name: "refactor",
        description: "Refactor code",
        emoji: "♻️ ",
    },
    StaticCategory {
        name: "perf",
        description: "Performance improvement",
        emoji: "⚡",
    },
    StaticCategory {
        name: "test",
        description: "Add or update tests",
        emoji: "🧪",
    },
    StaticCategory {
        name: "build",
        description: "Build system changes",
        emoji: "📦",
    },
    StaticCategory {
        name: "ci",
        description: "CI/CD changes",
        emoji: "⚙️ ",
    },
    StaticCategory {
        name: "chore",
        description: "Maintenance / chores",
        emoji: "🧹",
    },
    StaticCategory {
        name: "revert",
        description: "Revert a commit",
        emoji: "⏪",
    },
    StaticCategory {
        name: "wip",
        description: "Work in progress",
        emoji: "🚧",
    },
    StaticCategory {
        name: "none",
        description: "No category prefix",
        emoji: "── ",
    },
];
