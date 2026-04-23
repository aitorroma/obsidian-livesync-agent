# livesync-agent (Rust)

Binario headless para sincronizar un vault de Obsidian de forma bidireccional usando CouchDB.

> Estado: MVP funcional para agentes/automatización.
> Formatos soportados: `agent-file` y `plain` + `leaf` (LiveSync).

## Quickstart

```bash
cargo build --release
./target/release/livesync-agent setup --config ./livesync-agent.toml
./target/release/livesync-agent --config ./livesync-agent.toml sync-once
```

También puedes usar flags no interactivos:

```bash
./target/release/livesync-agent setup \
  --config ./livesync-agent.toml \
  --yes \
  --vault-path "$HOME/Obsidian" \
  --base-url "https://data.example.com" \
  --database "obsidian" \
  --username "admin"
```

## Instalación fácil (usuario)

Instala binario + config en `~/.livesync-agent/` y servicio `systemd --user`:

```bash
./scripts/install-user-service.sh \
  --vault-path "$HOME/Obsidian" \
  --base-url "https://data.example.com" \
  --database "obsidian" \
  --username "admin"
```

Si pasas `--username` y no pasas `--password`, pedirá password por prompt oculto.

Rutas creadas:

- Binario: `~/.local/bin/livesync-agent`
- Config: `~/.livesync-agent/config.toml`
- Servicio: `~/.config/systemd/user/livesync-agent.service`

Comandos útiles:

```bash
systemctl --user status livesync-agent.service
journalctl --user -u livesync-agent.service -f
systemctl --user restart livesync-agent.service
```

Desinstalar:

```bash
./scripts/uninstall-user-service.sh
```

## Instalación en servidor (systemd global)

Para servidores/headless:

```bash
sudo ./scripts/install-server-service.sh \
  --user "tuxed" \
  --vault-path "/home/tuxed/Obsidian" \
  --base-url "https://data.example.com" \
  --database "obsidian" \
  --username "admin"
```

Crea:

- Binario: `/usr/local/bin/livesync-agent`
- Config: `/etc/livesync-agent/<user>.toml`
- Servicio: `livesync-agent@<user>.service`

Logs/estado:

```bash
systemctl status livesync-agent@tuxed.service
journalctl -u livesync-agent@tuxed.service -f
```

## Uso manual

```bash
./target/release/livesync-agent --config ~/.livesync-agent/config.toml sync-once
./target/release/livesync-agent --config ~/.livesync-agent/config.toml daemon --interval-seconds 30
```

## Config (ejemplo)

```toml
vault_path = "/path/to/vault"
state_path = "/path/to/vault/.livesync-agent/state.json"
ignore_prefixes = [".git/", ".livesync-agent/"]

[couchdb]
base_url = "https://couchdb.example.com"
database = "obsidian"
username = "admin"
password = "secret"
```

## Qué hace

- `sync-once`: ciclo pull → push.
- `daemon`: sincronización periódica.
- reconstruye archivos locales desde remoto.
- sube cambios locales y borrados.
- guarda checkpoint en `.livesync-agent/state.json`.
