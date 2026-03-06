#[derive(Debug, Clone)]
pub struct Category {
    pub name: &'static str,
    pub description: &'static str,
    pub emoji: &'static str,
}

pub const CATEGORIES: &[Category] = &[
    Category {
        name: "feat",
        description: "Add a new feature",
        emoji: "✨",
    },
    Category {
        name: "fix",
        description: "Fix a bug",
        emoji: "🐛",
    },
    Category {
        name: "docs",
        description: "Documentation changes",
        emoji: "📚",
    },
    Category {
        name: "style",
        description: "Code style / formatting",
        emoji: "🎨",
    },
    Category {
        name: "refactor",
        description: "Refactor code",
        emoji: "♻️ ",
    },
    Category {
        name: "perf",
        description: "Performance improvement",
        emoji: "⚡",
    },
    Category {
        name: "test",
        description: "Add or update tests",
        emoji: "🧪",
    },
    Category {
        name: "build",
        description: "Build system changes",
        emoji: "📦",
    },
    Category {
        name: "ci",
        description: "CI/CD changes",
        emoji: "⚙️ ",
    },
    Category {
        name: "chore",
        description: "Maintenance / chores",
        emoji: "🧹",
    },
    Category {
        name: "revert",
        description: "Revert a commit",
        emoji: "⏪",
    },
    Category {
        name: "wip",
        description: "Work in progress",
        emoji: "🚧",
    },
    Category {
        name: "none",
        description: "No category prefix",
        emoji: "── ",
    },
];
