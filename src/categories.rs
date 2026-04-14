#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
    pub description: String,
}

/// Compile-time category definition (for the built-in defaults).
pub struct StaticCategory {
    pub name: &'static str,
    pub description: &'static str,
}

/// Built-in default categories (used when no config overrides them).
pub const DEFAULT_CATEGORIES: &[StaticCategory] = &[
    StaticCategory {
        name: "feat",
        description: "Add a new feature",
    },
    StaticCategory {
        name: "fix",
        description: "Fix a bug",
    },
    StaticCategory {
        name: "docs",
        description: "Documentation changes",
    },
    StaticCategory {
        name: "style",
        description: "Code style / formatting",
    },
    StaticCategory {
        name: "refactor",
        description: "Refactor code",
    },
    StaticCategory {
        name: "perf",
        description: "Performance improvement",
    },
    StaticCategory {
        name: "test",
        description: "Add or update tests",
    },
    StaticCategory {
        name: "build",
        description: "Build system changes",
    },
    StaticCategory {
        name: "ci",
        description: "CI/CD changes",
    },
    StaticCategory {
        name: "chore",
        description: "Maintenance / chores",
    },
    StaticCategory {
        name: "revert",
        description: "Revert a commit",
    },
    StaticCategory {
        name: "wip",
        description: "Work in progress",
    },
    StaticCategory {
        name: "none",
        description: "No category prefix",
    },
];
