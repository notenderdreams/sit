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

| Command | Description |
|---------|-------------|
| `sit commit` / `sit c` | interactive commit flow |
| `sit <category> [message...]` | category shortcut, e.g. `sit feat add search` |
| `sit branch` / `sit b` | switch or create branches with picker |
| `sit log` / `sit l` | show a graph view of recent commits |
| `sit push` / `sit p` | push current branch to upstream |
| `sit release` / `sit rel` | show previous release tag, prompt next version, create and push new tag |
| `sit amend` / `sit a` | amend last commit |
| `sit undo` / `sit u` | soft-reset last commit |
| `sit init` | scaffold `.sit/config.toml` and hook templates |

> NOTE: I genuinely wanted something like this for my own projects, so I built it. There are other tools out there that may do similar things, but I wanted something minimal and tailored to my workflow.
