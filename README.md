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

## Commands

- `sit commit` / `sit c`: interactive commit flow
- `sit branch` / `sit b`: switch or create branches with picker
- `sit log` / `sit l`: show a graph view of recent commits
- `sit push` / `sit p`: push current branch to upstream
- `sit release` / `sit rel`: show previous release tag, prompt next version, create and push new tag
- `sit amend` / `sit a`: amend last commit
- `sit undo` / `sit u`: soft-reset last commit
- `sit init`: scaffold `.sit/config.toml` and hook templates

## Config

- Global config: `~/sit.toml`
- Project config: `.sit/config.toml` (searched from current directory upward)
- Run `sit init` to scaffold `.sit/config.toml` and `.sit/hooks/*` template files with commented examples.

```rs
[commit]
template = "$type($mod): $message"
ask_description = true

[modules]
core     = "Core logic"
cli      = { desc = "CLI interface", path = "src/cmd" }
config   = "Configuration system"
ui       = { desc = "User interface", paths = ["src/ui", "src/style"] }
git      = "Git operations"

[categories]
feat = "Add a new feature"
fix = "Fix a bug"
docs = "Documentation changes"
style = "Code style / formatting"
refactor = "Refactor code"
perf = "Performance improvement"
test = "Add or update tests"
build = "Build system changes"
ci = "CI/CD changes"
chore = "Maintenance / chores"
revert = "Revert a commit"
wip = "Work in progress"
none = "No category prefix"
```

Module entries accept either a simple description string or an object with optional `path` / `paths` folder mappings. During commit, `sit` counts selected-file matches per module path and opens the module picker with the highest-match module preselected, so pressing Enter accepts it immediately.

For a full walkthrough of config locations, precedence, `.sit` initialization, sections, and hook env behavior, see `docs/config.md`.

> Note
> I genuinely wanted something like this for my projects, so I built it. There are other tools out there that do similar things, but I wanted something minimal and tailored to my workflow.
