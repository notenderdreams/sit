# sit (structured interactive git)

A minimal CLI tool for structured Git commits. It provides an interactive workflow to select changed files, pick a commit category (e.g., feat, fix, docs), and compose a formatted commit message — all from the terminal.

## Installation

### macOS / Linux

```bash
curl -fsSL https://raw.githubusercontent.com/notenderdreams/sit/main/installer/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/notenderdreams/sit/main/installer/install.ps1 | iex
```

### Local build

```bash
cargo install --path .
```

Then run:

```bash
sit --help
```

```rs
[commit]
template = "$type($mod): $message"
ask_description = true
show_emoji = true

[modules]
core     = "Core logic"
cli      = "CLI interface"
config   = "Configuration system"
ui       = "User interface"
git      = "Git operations"

[categories]
feat     = { emoji = "✨", desc = "Add a new feature" }
fix      = { emoji = "🐛", desc = "Fix a bug" }
docs     = { emoji = "📚", desc = "Documentation changes" }
style    = { emoji = "🎨", desc = "Code style / formatting" }
refactor = { emoji = "♻️",  desc = "Refactor code" }
perf     = { emoji = "⚡", desc = "Performance improvement" }
test     = { emoji = "🧪", desc = "Add or update tests" }
build    = { emoji = "📦", desc = "Build system changes" }
ci       = { emoji = "⚙️",  desc = "CI/CD changes" }
chore    = { emoji = "🧹", desc = "Maintenance / chores" }
revert   = { emoji = "⏪", desc = "Revert a commit" }
wip      = { emoji = "🚧", desc = "Work in progress" }
none     = { emoji = "──",  desc = "No category prefix" }
```

> Note
> I genuinely wanted something like this for my projects, so I built it. There are other tools out there that do similar things, but I wanted something minimal and tailored to my workflow.
