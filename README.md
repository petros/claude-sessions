# claude-sessions

![Rust action status](https://github.com/petros/drem/actions/workflows/rust.yml/badge.svg) [![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg?style=flat-square)](https://makeapullrequest.com)

A CLI tool for managing and organizing [Claude Code](https://claude.ai/code) sessions.

Map meaningful labels to session IDs, track multiple sessions per label, and resume your work with a simple command.

## Features

- **Label-based organization**: Map any label (ticket IDs, project names, feature branches) to Claude sessions
- **One-to-many mapping**: Each label can have multiple sessions with optional descriptions
- **Quick resume**: Jump back into any session with `claude-sessions resume <label>`
- **Interactive picker**: Choose from multiple sessions when needed
- **Directory aware**: Each session remembers its working directory

## Installation

### From source

```bash
git clone https://github.com/petros/claude-sessions.git
cd claude-sessions
cargo install --path .
```

### Manual build

```bash
cargo build --release
cp target/release/claude-sessions /usr/local/bin/
```

## Usage

### Add a session to a label

```bash
# Basic usage - saves current directory
claude-sessions add my-feature abc123-session-id

# With a description
claude-sessions add TICKET-123 abc123-session-id -d "Initial investigation"

# Add another session to the same label
claude-sessions add TICKET-123 def456-session-id -d "Follow-up with fix"
```

### Resume a session

```bash
# Resume the most recent session for a label
claude-sessions resume my-feature

# Pick from multiple sessions interactively
claude-sessions resume TICKET-123 --pick
```

### List labels and sessions

```bash
# List all labels
claude-sessions list

# Show sessions for a specific label
claude-sessions list TICKET-123
```

### Manage labels

```bash
# Set or update a label's description
claude-sessions describe TICKET-123 -d "Authentication bug in OAuth flow"

# Remove a specific session from a label
claude-sessions remove TICKET-123 abc123-session-id

# Remove an entire label and all its sessions
claude-sessions remove TICKET-123
```

### Configuration

```bash
# Show where data is stored
claude-sessions config
```

## Data Storage

Sessions are stored in a JSON file at:

- **macOS**: `~/Library/Application Support/claude-sessions/data.json`
- **Linux**: `~/.config/claude-sessions/data.json`

## Example Workflow

```bash
# Start working on a support ticket
cd ~/Projects/my-app
claude  # Start a new Claude session

# Claude shows session ID, map it to your ticket
claude-sessions add FRONT-12345 a1b2c3-session-id -d "Customer reports login issues"

# Later, need to continue the investigation
claude-sessions resume FRONT-12345

# Started a new session for the same ticket
claude-sessions add FRONT-12345 d4e5f6-session-id -d "Found root cause, implementing fix"

# See all sessions for the ticket
claude-sessions list FRONT-12345
```

## Development

This project uses [just](https://github.com/casey/just) as a command runner (optional).

```bash
# Install just (macOS)
brew install just

# List all available recipes
just

# Common commands
just build    # Build debug binary
just release  # Build release binary
just check    # Run fmt, lint, and tests
just install  # Install locally
```

### Creating a Release

Releases are automated via GitHub Actions. To create a new release:

1. Update the version in `Cargo.toml`
2. Commit and tag:
   ```bash
   git add Cargo.toml
   git commit -m "Bump version to X.Y.Z"
   git tag vX.Y.Z
   git push origin main --tags
   ```

The workflow will automatically build binaries for all platforms and create a GitHub release with checksums.
