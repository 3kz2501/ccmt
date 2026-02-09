# ccm

AI-powered commit message generator using Claude.

Generates meaningful commit messages from your staged git diff using Claude CLI or the Anthropic API.

[日本語ドキュメント](docs/readme.ja.md)

## Features

- **Two providers** - Claude CLI (subscription) or Anthropic HTTP API
- **Conventional Commits** - Automatic `type(scope): description` formatting
- **Gitmoji** support - Optional emoji prefixes
- **Multi-language** - Generate messages in English, Japanese, or any language
- **Interactive confirmation** - Yes / Edit / Cancel before committing
- **Edit loop** - Describe revisions and let Claude regenerate
- **Layered config** - Global + project-level + CLI flags
- **Git hook** - Auto-generate messages via `prepare-commit-msg`
- **Dry-run mode** - Preview messages without committing

## Installation

### From crates.io

```bash
cargo install ccm
```

### From source

```bash
git clone https://github.com/kz/ccm.git
cd ccm
cargo install --path .
```

### From GitHub Releases

Download the pre-built binary for your platform from the [Releases](https://github.com/kz/ccm/releases) page.

## Prerequisites

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install) (for building from source / `cargo install`)

Plus one of the following, depending on your provider:

| Provider | Requirement |
|----------|-------------|
| `cli` (default) | [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) installed and authenticated |
| `api` | [Anthropic API key](https://console.anthropic.com/) |

## Quick Start

```bash
# Stage your changes
git add .

# Generate and commit
ccm

# Or preview without committing
ccm --dry-run
```

## Usage

```
USAGE: ccm [OPTIONS] [COMMAND]

COMMANDS:
  config init     Generate default global config
  config show     Show merged configuration
  hook install    Install prepare-commit-msg hook
  hook remove     Remove installed hook

OPTIONS:
  -m, --message <HINT>     Context hint for the AI
  -d, --dry-run            Generate message only, don't commit
      --push               Push after commit
      --no-confirm         Skip confirmation prompt
      --provider <cli|api> Override auth provider
      --model <MODEL>      Override model (sonnet, haiku, opus, or full ID)
      --language <LANG>    Override language (en, ja, etc.)
  -h, --help               Print help
```

### Examples

```bash
# Basic usage
ccm

# With context hint
ccm -m "refactored auth module for better testability"

# Japanese commit messages
ccm --language ja

# Use API instead of CLI
ccm --provider api

# Dry-run with a specific model
ccm --dry-run --model haiku

# Auto-push after commit
ccm --push

# Non-interactive (CI/scripts)
ccm --no-confirm
```

## Confirmation Flow

When `confirm = true` (default), ccm presents an interactive prompt:

```
Generated commit message:
────────────────────────────────────────
feat(auth): add JWT token validation

Implement login/logout endpoints and
middleware for token verification.
────────────────────────────────────────

? Commit with this message?
> Yes  - Commit with this message
  Edit - Revise the message
  No   - Cancel
```

Selecting **Edit** lets you describe what to change. Claude regenerates the message based on your instruction, and the loop repeats until you confirm or cancel.

## Configuration

ccm works with zero configuration using sensible defaults. Optionally customize via config files.

### Generate Config

```bash
ccm config init    # Creates ~/.config/ccm/config.toml
ccm config show    # Shows the merged config
```

### Global Config (`~/.config/ccm/config.toml`)

```toml
[auth]
provider = "cli"        # "cli" (Claude CLI) or "api" (HTTP API)
api_key = ""            # Required for "api" provider
model = "sonnet"        # "sonnet", "haiku", "opus", or full model ID

[commit]
conventional = true     # Conventional Commits format
emoji = false           # Gitmoji prefixes
language = "en"         # Message language ("en", "ja", etc.)
auto_stage = false      # Run `git add .` before generating
auto_push = false       # Run `git push` after committing
confirm = true          # Interactive confirmation prompt

[prompt]
system = ""             # Custom system prompt (appended to default)
max_diff_length = 8000  # Max diff characters sent to AI
```

### Project Config (`.ccm.toml`)

Place a `.ccm.toml` in your project root (or any parent directory) to override settings per-project. Only include the fields you want to override:

```toml
[commit]
language = "ja"
conventional = true
```

### Config Priority

Settings are merged in this order (later wins):

1. Hardcoded defaults
2. `~/.config/ccm/config.toml` (global)
3. `.ccm.toml` (project, searched from cwd upward)
4. CLI flags (highest priority)

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ANTHROPIC_API_KEY` | Fallback API key when `auth.api_key` is empty |

## Providers

### `cli` (default)

Uses the [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) as a subprocess. Requires Claude CLI to be installed and authenticated.

```bash
ccm --provider cli
```

### `api`

Direct HTTP calls to the Anthropic Messages API. Requires an API key.

```bash
# Via config
ccm config init
# Edit ~/.config/ccm/config.toml: provider = "api", api_key = "sk-ant-..."

# Or via environment variable
export ANTHROPIC_API_KEY="sk-ant-..."
ccm --provider api

# Or one-off
ccm --provider api  # Uses ANTHROPIC_API_KEY env var
```

### Model Aliases

| Alias | Model ID |
|-------|----------|
| `sonnet` | `claude-sonnet-4-5-20250929` |
| `haiku` | `claude-haiku-4-5-20251001` |
| `opus` | `claude-opus-4-6` |

You can also pass a full model ID: `--model claude-sonnet-4-5-20250929`

## Git Hook

Install a `prepare-commit-msg` hook to auto-generate messages for `git commit`:

```bash
ccm hook install    # Installs to .git/hooks/prepare-commit-msg
ccm hook remove     # Removes (restores backup if exists)
```

With the hook installed, running `git commit` will automatically populate the commit message using ccm. The hook is skipped for merge and squash commits.

## Project Structure

```
src/
├── main.rs              # Entry point, main flow, confirmation loop
├── cli.rs               # CLI argument definitions (clap)
├── config.rs            # Config loading, merging, init/show
├── git.rs               # Git subprocess operations
├── prompt.rs            # AI prompt construction
├── format.rs            # Response cleanup and formatting
├── hook.rs              # Git hook install/remove
└── provider/
    ├── mod.rs           # Provider trait and factory
    ├── claude_cli.rs    # Claude CLI subprocess provider
    └── api.rs           # Anthropic HTTP API provider
```

## License

MIT
