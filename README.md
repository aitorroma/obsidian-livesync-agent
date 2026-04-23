<div align="center">

  <a href="https://t.me/aitorroma">
    <img src="https://tva1.sinaimg.cn/large/008i3skNgy1gq8sv4q7cqj303k03kweo.jpg" alt="Aitor Roma" />
  </a>

  <br>

  [![ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/J3J64AN17)

  <br>

  <a href="https://t.me/aitorroma">
    <img src="https://img.shields.io/badge/Telegram-informational?style=for-the-badge&logo=telegram&logoColor=white" alt="Telegram Badge"/>
  </a>
</div>

# livesync-agent

Headless agent to sync Obsidian vaults bidirectionally using CouchDB (Linux/macOS).

> 🇪🇸 Spanish docs: [README_es.md](./README_es.md)

## 0) Install the Obsidian plugin first

This agent is designed to work with **Self-hosted LiveSync**.

Install plugin in Obsidian from:
- https://obsidian.md/es/plugins?search=Self-hosted%20LiveSync

## 1) Deploy CouchDB with HTTPS (Traefik + Swarm)

Use the included stack:
- `deploy/couchdb.yaml`
- `deploy/README.md`

Quick deploy:

```bash
printf 'admin' | docker secret create couchdb_user -
printf 'CHANGE_ME_STRONG_PASSWORD' | docker secret create couchdb_password -
docker stack deploy -c deploy/couchdb.yaml couchdb
```

## 2) Install livesync-agent

### Option A: release installer script (Linux x86_64 + macOS Intel/Apple Silicon)

```bash
curl -fsSL https://raw.githubusercontent.com/aitorroma/obsidian-livesync-agent/main/scripts/install.sh | bash
```

### Option B: Homebrew

Tap and install:

```bash
brew tap aitorroma/obsidian-livesync-agent https://github.com/aitorroma/obsidian-livesync-agent
brew install livesync-agent
```

Install latest `main` version:

```bash
brew install --HEAD aitorroma/obsidian-livesync-agent/livesync-agent
```

### Option C: from source

```bash
cargo build --release
./target/release/livesync-agent --help
```

## 3) Configure

Interactive setup:

```bash
livesync-agent setup
```

Default config path: `~/.livesync-agent/config.toml` (directory is created automatically).

Then run one cycle:

```bash
livesync-agent sync-once
```

Or daemon mode:

```bash
livesync-agent daemon --interval-seconds 30
```

## Releases

GitHub Actions release workflow:
- File: `.github/workflows/release.yml`
- Trigger: push tag `v*` (example: `v0.1.1`)
- Build targets: **macOS Intel + Apple Silicon**

Create release:

```bash
git tag v0.1.1
git push origin v0.1.1
```

Assets uploaded:
- `livesync-agent-<tag>-x86_64-apple-darwin.tar.gz`
- `livesync-agent-<tag>-aarch64-apple-darwin.tar.gz`
- `SHA256SUMS`
