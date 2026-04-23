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

Agente headless para Linux que sincroniza vaults de Obsidian en modo bidireccional usando CouchDB.

> 🇬🇧 Documentación en inglés: [README.md](./README.md)

## 0) Instala primero el plugin de Obsidian

Este agente está pensado para funcionar con **Self-hosted LiveSync**.

Instalación del plugin:
- https://obsidian.md/es/plugins?search=Self-hosted%20LiveSync

## 1) Despliega CouchDB con HTTPS (Traefik + Swarm)

Usa los ficheros incluidos:
- `deploy/couchdb.yaml`
- `deploy/README.md`

Despliegue rápido:

```bash
printf 'admin' | docker secret create couchdb_user -
printf 'CAMBIA_A_PASSWORD_SEGURA' | docker secret create couchdb_password -
docker stack deploy -c deploy/couchdb.yaml couchdb
```

## 2) Instalar livesync-agent

### Opción A: script de instalación por release (Linux x86_64)

```bash
curl -fsSL https://raw.githubusercontent.com/aitorroma/obsidian-livesync-agent/main/scripts/install.sh | bash
```

### Opción B: Homebrew

```bash
brew tap aitorroma/obsidian-livesync-agent https://github.com/aitorroma/obsidian-livesync-agent
brew install livesync-agent
```

Para instalar la versión más reciente de `main`:

```bash
brew install --HEAD aitorroma/obsidian-livesync-agent/livesync-agent
```

### Opción C: compilar desde código fuente

```bash
cargo build --release
./target/release/livesync-agent --help
```

## 3) Configuración

Setup interactivo:

```bash
livesync-agent setup --config ~/.livesync-agent/config.toml
```

Ejecutar un ciclo:

```bash
livesync-agent --config ~/.livesync-agent/config.toml sync-once
```

Modo daemon:

```bash
livesync-agent --config ~/.livesync-agent/config.toml daemon --interval-seconds 30
```

## Releases

Workflow de release en GitHub Actions:
- Fichero: `.github/workflows/release.yml`
- Trigger: push de tag `v*` (ejemplo: `v0.1.1`)
- Build target: **solo Linux x86_64**

Crear una release:

```bash
git tag v0.1.1
git push origin v0.1.1
```

Assets publicados:
- `livesync-agent-<tag>-x86_64-unknown-linux-gnu.tar.gz`
- `SHA256SUMS`
