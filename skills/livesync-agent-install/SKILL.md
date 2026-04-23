---
name: livesync-agent-install
description: >
  Installs and configures livesync-agent for operators and automation agents.
  Trigger: When asked to install, update, or bootstrap livesync-agent on Linux/macOS.
license: Apache-2.0
metadata:
  author: aitorroma
  version: "1.0"
---

## When to Use

- User asks to install `livesync-agent` on a server/workstation.
- User asks to update an existing installation.
- User needs quick bootstrap for sync (`setup`, `sync-once`, `daemon`).

## Critical Patterns

1. **Plugin prerequisite first**
   - Confirm Obsidian plugin requirement: Self-hosted LiveSync.
   - Link: https://obsidian.md/es/plugins?search=Self-hosted%20LiveSync

2. **Platform decision**
   - **Linux x86_64**: Prefer release installer script.
   - **macOS**: Prefer Homebrew (source build).

3. **Default config path**
   - Always use `~/.livesync-agent/config.toml` unless user requests override.
   - `setup` should be run before daemon in fresh installs.

4. **Validation after install**
   - Always run:
     - `livesync-agent --version`
     - `livesync-agent setup` (or non-interactive equivalent)
     - `livesync-agent sync-once`

## Installation Playbook

### Linux x86_64

```bash
curl -fsSL https://raw.githubusercontent.com/aitorroma/obsidian-livesync-agent/main/scripts/install.sh | bash
livesync-agent --version
livesync-agent setup
livesync-agent sync-once
```

### macOS (Homebrew)

```bash
brew tap aitorroma/obsidian-livesync-agent
brew install livesync-agent
livesync-agent --version
livesync-agent setup
livesync-agent sync-once
```

### Update Commands

Linux (reinstall latest release binary):

```bash
curl -fsSL https://raw.githubusercontent.com/aitorroma/obsidian-livesync-agent/main/scripts/install.sh | bash
```

Homebrew update:

```bash
brew update
brew untap aitorroma/obsidian-livesync-agent || true
brew tap aitorroma/obsidian-livesync-agent
brew upgrade livesync-agent
```

## Commands

```bash
# First-time interactive config (default path: ~/.livesync-agent/config.toml)
livesync-agent setup

# One sync cycle
livesync-agent sync-once

# Continuous mode
livesync-agent daemon --interval-seconds 30
```

## Resources

- **Project docs (EN)**: [`README.md`](../../README.md)
- **Project docs (ES)**: [`README_es.md`](../../README_es.md)
- **CouchDB HTTPS deploy**: [`deploy/README.md`](../../deploy/README.md)
