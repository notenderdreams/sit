# Configuration Guide

This document explains how `sit` loads configuration, how to initialize a project with `.sit/`, and how each config section affects behavior.

## Config locations

`sit` supports both global and project-local config.

1. Global config (user-wide)
- Primary: `~/sit.toml`
- Legacy fallback: `~/.sitrc`

2. Local config (project-specific)
- Primary: `.sit/config.toml`
- Legacy fallback: `.sitrc`

For local config, `sit` searches from the current directory upward until it finds one.

## Merge and precedence rules

Configuration is built in this order:

1. Start with built-in defaults
2. Apply global config (if found)
3. Apply local config (if found)

Local values override global values.

Section behavior:
- `[categories]`: replaces the full category list if present
- `[modules]`: replaces the full module list if present

## Initialize `.sit` in a repository

Run:

```bash
sit init
```

This creates:

- `.sit/config.toml`
- `.sit/hooks/pre-commit`
- `.sit/hooks/post-commit`
- `.sit/hooks/pre-push`
- `.sit/hooks/post-push`

If a legacy `.sitrc` exists in the current directory, `sit init` migrates it to `.sit/config.toml`.

## `[commit]` section

Controls commit message formatting and commit flow prompts.

Example:

```toml
[commit]
template = "$type($mod): $message"
ask_description = true
auto_push = false
```

Fields:
- `template`: subject format
  - placeholders: `$type`, `$mod`, `$message`
- `ask_description`: prompt for optional commit body
- `auto_push`: push automatically after successful commit/amend

Template behavior notes:
- If no module is chosen, `$mod` is removed.
- Common wrappers like `($mod)` are collapsed automatically.
- If category is `none`, subject becomes either:
  - `(<module>): <message>` if module exists
  - `<message>` otherwise

## `[categories]` section

Defines available commit categories.

You can use either shorthand strings or table values.

Shorthand form:

```toml
[categories]
feat = "Add a new feature"
fix = "Fix a bug"
```

Table form:

```toml
[categories]
feat = { desc = "Add a new feature" }
fix = { desc = "Fix a bug" }
```

If `[categories]` is present, it replaces defaults completely.

## `[modules]` section

Defines available modules shown during commit.

Shorthand form:

```toml
[modules]
core = "Core logic"
```

Object form with folder mapping:

```toml
[modules]
ui  = { desc = "User interface", paths = ["src/ui", "src/style"] }
git = { desc = "Git operations", path = "src/git" }
```

Supported keys in module objects:
- `desc` (alias: `description`)
- `path` (aliases: `folder`, `folder_path`)
- `paths` (alias: `folders`)

Path rules:
- Leading `./` and trailing `/` are normalized away.
- A file matches a module when file path equals the module path prefix or starts with `<module_path>/`.
- Rename entries like `old/path -> new/path` are handled.

Auto module preselection:
- During commit, `sit` scores selected files against module paths.
- The highest-scoring module is preselected in the module picker.
- On ties, the first module in config order wins.

## Hook templates and runtime hook env

`sit` executes hooks from `.sit/hooks/` if they exist.

Pre hooks abort operation on non-zero exit:
- `pre-commit`
- `pre-push`

Post hooks warn on non-zero exit but do not abort:
- `post-commit`
- `post-push`

Built-in runtime environment variables passed to hooks include:

Commit hooks:
- `SIT_CATEGORY`
- `SIT_MESSAGE`
- `SIT_FILES`

Push hooks:
- `SIT_REMOTE`
- `SIT_BRANCH`
- `SIT_FORCE` (force push flow)

## Passing extra env vars from CLI

You can pass additional env vars to hooks with repeatable `--env`.

Examples:

```bash
sit commit --env APP_ENV=dev --env TEAM=platform
sit push --env RELEASE=1
sit feat --env SKIP_TEST=1 "ship API"
```

Rules:
- Format must be `KEY=VALUE`
- Repeatable flag; last value wins on duplicate keys
- `SIT_*` keys are reserved and cannot be overridden

The `--env` parser also accepts `--env=KEY=VALUE` in shortcut arg parsing.

## Typical setup patterns

1. Keep defaults globally, customize per project locally
- Put common template settings in `~/sit.toml`
- Put project categories/modules in `.sit/config.toml`

2. Team-wide module mapping
- Define module `paths` once in project config
- Let auto module selection prefill picker

3. Fast local commits with optional skip logic in hooks
- Put checks in `.sit/hooks/pre-commit`
- Use a guard env like `SKIP_TEST=1` when explicitly needed

## Troubleshooting

1. Config not applied
- Check current working directory
- Verify `.sit/config.toml` exists in current tree
- Confirm TOML is valid

2. Wrong module preselected
- Check module `path`/`paths` prefixes
- Ensure selected files match expected folders
- Remember tie-break uses config order

3. Hook env not visible
- Ensure hook file exists at `.sit/hooks/<name>`
- Print env inside hook script for debugging
- Confirm `--env KEY=VALUE` format is correct
