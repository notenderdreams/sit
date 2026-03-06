# sit (structured interactive git)

A minimal CLI tool for structured Git commits. It provides an interactive workflow to select changed files, pick a commit category (e.g., feat, fix, docs), and compose a formatted commit message — all from the terminal.

```rs
pub const CATEGORIES: &[Category] = &[
    Category { name: "feat",     description: "Add a new feature",       emoji: "✨" },
    Category { name: "fix",      description: "Fix a bug",               emoji: "🐛" },
    Category { name: "docs",     description: "Documentation changes",   emoji: "📚" },
    Category { name: "style",    description: "Code style / formatting", emoji: "🎨" },
    Category { name: "refactor", description: "Refactor code",           emoji: "♻️ " },
    Category { name: "perf",     description: "Performance improvement", emoji: "⚡" },
    Category { name: "test",     description: "Add or update tests",     emoji: "🧪" },
    Category { name: "build",    description: "Build system changes",    emoji: "📦" },
    Category { name: "ci",       description: "CI/CD changes",           emoji: "⚙️ " },
    Category { name: "chore",    description: "Maintenance / chores",    emoji: "🧹" },
    Category { name: "revert",   description: "Revert a commit",         emoji: "⏪" },
    Category { name: "wip",      description: "Work in progress",        emoji: "🚧" },
    Category { name: "none",     description: "No category prefix",      emoji: "── " },
];
```

> Note 
> I genuinely wanted something like this for my projects, so I built it. There are other tools out there that do similar things, but I wanted something minimal and tailored to my workflow. 