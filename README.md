# livesync-agent (Rust)

Binario headless para sincronizar un vault de Obsidian en modo bidireccional usando CouchDB.

> Estado actual: MVP funcional para agentes/automatización.
> Soporta formato `agent-file` y también `plain` + `leaf` (LiveSync).

## Instalación fácil (recomendada)

Instala binario + config en `~/.livesync-agent/` + servicio `systemd --user`:

```bash
./rust/livesync-agent/scripts/install-user-service.sh \
  --vault-path "$HOME/Obsidian" \
  --base-url "https://data.example.com" \
  --database "obsidian" \
  --username "admin"
```

> Si usas usuario/password, el script te pide la contraseña por prompt (sin mostrarla).

Queda así:

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
./rust/livesync-agent/scripts/uninstall-user-service.sh
```


## Instalación en servidor (systemd global)

Para servidores sin sesión de escritorio, instala como servicio de sistema (requiere `sudo`):

```bash
sudo ./rust/livesync-agent/scripts/install-server-service.sh \
  --user "tuxed" \
  --vault-path "/home/tuxed/Obsidian" \
  --base-url "https://data.example.com" \
  --database "obsidian" \
  --username "admin"
```

Esto crea:

- binario: `/usr/local/bin/livesync-agent`
- config: `/etc/livesync-agent/<user>.toml`
- servicio: `livesync-agent@<user>.service`

Logs/estado:

```bash
systemctl status livesync-agent@tuxed.service
journalctl -u livesync-agent@tuxed.service -f
```

## Uso manual

```bash
./rust/livesync-agent/target/release/livesync-agent --config ~/.livesync-agent/config.toml sync-once
./rust/livesync-agent/target/release/livesync-agent --config ~/.livesync-agent/config.toml daemon --interval-seconds 30
```

## Config

Ejemplo:

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
- recrea archivos locales desde remoto.
- sube cambios locales y tombstones de borrado.
- mantiene checkpoint en `.livesync-agent/state.json`.

## Próximos pasos para compatibilidad LiveSync completa

- entender/reimplementar formato real completo de `EntryDoc`/chunks del commonlib.
- cifrado compatible con el plugin (PBKDF2/salt + flujo de claves).
- estrategia de conflictos equivalente.
- soporte de hidden/config sync del ecosistema LiveSync.
