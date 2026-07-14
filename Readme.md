# clicense

A CLI tool and API server for managing open source license templates, written in Rust.

**clicense** helps you quickly create license files for your projects. It ships with 10 popular open source license templates built-in, supports custom licenses with rich metadata (permissions, conditions, limitations, custom key-value fields), online updates via `.zip` export, and a configurable default system that lets you generate a LICENSE file with a single command. The companion **clicense-server** provides a REST API and systemd service for self-hosting.

## Features

- **10 built-in license templates** — MIT, Apache-2.0, GPL-3.0, LGPL-3.0, BSD-3-Clause, BSD-2-Clause, MPL-2.0, Unlicense, ISC, EPL-2.0
- **Custom licenses with metadata** — Add your own templates with display name, description, SPDX identifier, permissions, conditions, limitations, keywords, and arbitrary custom key-value fields
- **`.zip` export / import** — Bundle all licenses (templates + metadata) into a single `.zip` for distribution or backup
- **Configurable defaults** — Set default author, year, and license so `clicense new` just works
- **Online updates with diff** — Pull `.zip` exports from a remote server, see what changed before writing
- **Remote browsing** — List, inspect, and download licenses directly from a remote server
- **Systemd service** — `clicense-server service install` deploys as a systemd unit with security hardening
- **Config directory resolution** — `/etc/clicense-server/` (system) → `~/.clicense-server/` (user), overridable via `--config-dir`

## Installation

### From source

```bash
git clone https://github.com/aplt-cnt/cnt-license.git
cd cnt-license

# CLI only
cargo install --path .

# Server
cargo install --path server
```

### Debian package (Linux)

```bash
# Build on Linux:
cargo build --release -p cnt-license -p clicense-server
cargo deb -p clicense-server
cargo deb -p cnt-license

# Install:
sudo dpkg -i target/debian/clicense-server_1.1.0-1_amd64.deb \
             target/debian/cnt-license_1.1.0-1_amd64.deb
```

The `.deb` automatically initializes built-in templates and installs the systemd service.

## Quick Start

```bash
# Generate a MIT license file
clicense new mit -a "Your Name" -y 2026

# Set defaults, then one-command generate
clicense config default_author "Your Name"
clicense config default_license mit
clicense new

# List all available licenses
clicense list

# View detailed info (permissions / conditions / limitations)
clicense list gpl-3.0

# Output license text to stdout (with placeholder substitution)
clicense source mit -a "Your Name" -y 2026
```

## CLI Reference (`clicense`)

### `clicense new [license_id]`

Generate a license file.

| Option | Short | Description |
|--------|-------|-------------|
| `license_id` | — | License identifier (e.g. `mit`, `apache-2.0`). Falls back to `default_license` config |
| `--author` | `-a` | Copyright holder. Falls back to `default_author` config |
| `--year` | `-y` | Copyright year. Falls back to `default_year` config, then current year |
| `--output` | `-o` | Output file name. Falls back to `output_name` config, then `LICENSE` |

### `clicense config [key] [value]`

Manage configuration values.

```bash
clicense config --list                     # Show all keys with current values
clicense config default_author             # View a single key
clicense config default_author "TaimWay"   # Set a value
clicense config --reset default_author     # Reset to default
```

| Config key | Description | Default |
|------------|-------------|---------|
| `update_url` | Remote server URL for updates | `https://api.clicense.top` |
| `output_name` | Default output file name | `LICENSE` |
| `default_author` | Default copyright holder | *(not set)* |
| `default_year` | Default copyright year | *(current year)* |
| `default_license` | Default license identifier | *(not set)* |

Config stored at `~/.clicense/config.yml`.

### `clicense add <file> --name <id> [options]`

Add a custom license template with optional metadata.

| Option | Description |
|--------|-------------|
| `--name` | License identifier **(required)** |
| `--force` | Overwrite existing license |
| `--display-name` | Human-readable name (defaults to `--name`) |
| `--description` | License description |
| `--spdx-id` | SPDX identifier (defaults to `--name`) |
| `--permissions` | Repeatable, e.g. `--permissions "Commercial use"` |
| `--conditions` | Repeatable |
| `--limitations` | Repeatable |
| `--keywords` | Repeatable |
| `--custom` | Key=value pairs, e.g. `--custom url=https://example.com` |

Metadata is stored at `~/.clicense/meta/<id>.meta.toml`.

### `clicense update [--update-url URL]`

Download the latest `.zip` export from a remote server. Compares against local files and shows a diff summary:

- **ADDED** — New on server
- **UPDATED** — Content differs from local copy
- **UNCHANGED** — Identical, skipped
- **LOCAL ONLY** — Only exists locally, preserved

Templates are written to `~/.clicense/licenses/`, metadata to `~/.clicense/meta/`.

### `clicense remove <names...> [--all]`

Remove custom license templates and their metadata files.

### `clicense list [name] [--builtin|--custom]`

List installed licenses. Without a name, shows all licenses in a table. With a name, shows detailed three-column info (Permissions / Conditions / Limitations) plus custom fields.

### `clicense source <name> [-a author] [-y year]`

Output the raw content of a license to stdout. Supports `{year}` and `{author}` placeholder substitution.

### `clicense online [--online-url URL] <subcommand>`

Interact with a remote license server.

| Subcommand | Description |
|------------|-------------|
| `list` | List all licenses available on the server |
| `license <name>` | Show detailed info for a remote license |
| `source <name>` | Output raw license text from the server |

### `clicense version`

Show the current version.

---

## Server Reference (`clicense-server`)

### Server management

```bash
# Initialize (creates config, templates, meta dirs)
clicense-server init [--licenses-dir <path>] [--meta-dir <path>] [--force]

# Start the API server
clicense-server run [--host 0.0.0.0] [--port 3000] [--licenses-dir <path>] [--meta-dir <path>]

# View/set configuration
clicense-server config --list
clicense-server config host 0.0.0.0
clicense-server config --reset port
```

### License data management

```bash
# Add a custom license with full metadata
clicense-server add ./my-license.txt --name custom \
    --display-name "My Custom License" \
    --description "A custom license for my project" \
    --spdx-id "CUSTOM-1" \
    --permissions "Commercial use" --permissions "Distribution" \
    --conditions "License notice" \
    --limitations "Liability" \
    --keywords "custom" --keywords "permissive" \
    --custom url=https://example.com --custom version=2.0

# List all licenses
clicense-server list

# Show license detail with metadata
clicense-server list custom

# Output raw license content
clicense-server source mit

# Export all licenses to .zip
clicense-server export --output backup.zip

# Import licenses from .zip
clicense-server import backup.zip [--force]

# Clone from a remote server
clicense-server clone https://api.clicense.top

# Remove licenses
clicense-server remove custom [--all]
```

### Systemd service

```bash
# Install as systemd service (requires root)
clicense-server service install    # config at /etc/clicense-server/
clicense-server service start
clicense-server service stop
clicense-server service restart
clicense-server service enable
clicense-server service disable
clicense-server service status
clicense-server service reload     # systemctl daemon-reload
clicense-server service uninstall

# Manage licenses through the service
clicense-server service add ./my-license.txt --name custom --display-name "Custom"
clicense-server service list
clicense-server service source mit
clicense-server service export --output backup.zip
clicense-server service import backup.zip
clicense-server service clone https://api.clicense.top
```

Service data commands use `/etc/clicense-server/` as default config directory. Override with global `--config-dir`.

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/health` | Health check |
| `GET` | `/api/v1/version` | Server version |
| `GET` | `/api/v1/licenses` | List all license templates (YAML/JSON) |
| `GET` | `/api/v1/licenses/{id}` | Get a single license template |
| `GET` | `/api/v1/licenses/{id}/info` | Get license metadata (JSON) |
| `GET` | `/api/v1/search?q=` | Fuzzy search licenses by id/name/description/keywords |
| `GET` | `/api/v1/export` | Export all licenses + metadata as `.zip` |

Backward-compatible aliases (`/health`, `/licenses`, `/search`, `/export`, etc.) are also registered.

Default address: `http://0.0.0.0:3000`.

## Built-in Licenses

| ID | Full Name | SPDX |
|----|-----------|------|
| `mit` | MIT License | MIT |
| `apache-2.0` | Apache License 2.0 | Apache-2.0 |
| `gpl-3.0` | GNU General Public License v3.0 | GPL-3.0-only |
| `lgpl-3.0` | GNU Lesser General Public License v3.0 | LGPL-3.0-only |
| `bsd-3-clause` | BSD 3-Clause License | BSD-3-Clause |
| `bsd-2-clause` | BSD 2-Clause License | BSD-2-Clause |
| `mpl-2.0` | Mozilla Public License 2.0 | MPL-2.0 |
| `unlicense` | The Unlicense | Unlicense |
| `isc` | ISC License | ISC |
| `epl-2.0` | Eclipse Public License 2.0 | EPL-2.0 |

## Configuration

### clicense (`~/.clicense/config.yml`)

```yaml
update_url: https://api.clicense.top
output_name: LICENSE
default_author: ~
default_year: ~
default_license: ~
```

Custom licenses: `~/.clicense/licenses/<name>`
Custom metadata: `~/.clicense/meta/<name>.meta.toml`

### clicense-server (`/etc/clicense-server/config.yml` or `~/.clicense-server/config.yml`)

```yaml
host: 0.0.0.0
port: 3000
licenses_dir: /var/lib/clicense-server/licenses
meta_dir: /var/lib/clicense-server/meta
log_level: info
access_log: true
```

Config directory resolution:
1. `--config-dir <path>` CLI argument
2. `CLICENSE_SERVER_CONFIG_DIR` environment variable
3. `/etc/clicense-server/` (if exists)
4. `~/.clicense-server/` (fallback)

## Metadata format (`.meta.toml`)

```toml
name = "MIT License"
description = "A permissive license that is short and to the point."
spdx_id = "MIT"
permissions = ["Commercial use", "Distribution", "Modification", "Private use"]
conditions = ["License and copyright notice"]
limitations = ["Liability", "Warranty"]
keywords = ["permissive", "popular", "simple"]
placeholders = ["year", "author"]

[custom]
url = "https://opensource.org/licenses/MIT"
version = "1.0"
```

## Project Structure

```
cnt-license/                         Cargo workspace root
├── Cargo.toml                      Workspace config
├── Readme.md
├── LICENSE
├── licenses/                       10 built-in license templates (*.txt)
├── licenses-meta.toml              Built-in license metadata
├── client/                         CLI client crate (clicense binary)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 Entry point, clap command definitions
│       ├── config.rs               Configuration management
│       ├── license.rs              Built-in templates (LazyLock + include_str!)
│       ├── metadata.rs             Metadata loader + custom meta support
│       ├── http.rs                 Shared HTTP client (ureq)
│       └── command/
│           ├── mod.rs, add.rs, new.rs, list.rs, remove.rs
│           ├── online.rs, source.rs, update.rs, config_cmd.rs
└── server/                         API server crate (clicense-server binary)
    ├── Cargo.toml
    ├── packaging/
    │   ├── assets/                 systemd unit + default config
    │   └── deb-scripts/            postinst, prerm, postrm
    └── src/
        ├── main.rs                 Entry point + dispatch
        ├── cli.rs                  Clap command definitions
        ├── config.rs               ServerConfig with meta_dir
        ├── state.rs                AppState (templates + meta)
        ├── http.rs                 Shared HTTP client
        ├── data/
        │   ├── mod.rs              Template/meta loader, build_zip()
        │   └── licenses.toml       Built-in license metadata
        ├── handlers/
        │   ├── mod.rs, health.rs, version.rs
        │   ├── licenses.rs         GET /licenses, /licenses/{id}, /licenses/{id}/info
        │   ├── search.rs           GET /search
        │   └── export.rs           GET /export (.zip)
        ├── models/
        │   └── license.rs          LicenseMeta, SearchResponse, ErrorResponse
        └── command/
            ├── mod.rs, init.rs, run.rs, config_cmd.rs
            ├── add.rs, remove.rs, list.rs, clone.rs
            ├── source.rs, export.rs, import.rs
            └── service.rs          Systemd lifecycle + data management
```

## Tech Stack

**CLI**:
- [Rust](https://www.rust-lang.org/) (edition 2024)
- [clap](https://docs.rs/clap) 4.6 — command-line argument parser (derive)
- [ureq](https://docs.rs/ureq) 3 — HTTP client
- [serde](https://docs.rs/serde) + [serde_yaml](https://docs.rs/serde_yaml) + [toml](https://docs.rs/toml) — serialization
- [zip](https://docs.rs/zip) 2 — `.zip` archive read/write
- [colored](https://docs.rs/colored) — terminal colors
- [dirs](https://docs.rs/dirs) 6 — platform-specific directories

**Server**:
- [Axum](https://docs.rs/axum) 0.8 — web framework
- [Tokio](https://docs.rs/tokio) 1 — async runtime
- [tower-http](https://docs.rs/tower-http) — CORS middleware
- [tracing](https://docs.rs/tracing) — structured logging
- [zip](https://docs.rs/zip) 2 — `.zip` stream for `/api/v1/export`

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

---

<p align="center">
  <strong>clicense</strong> by APLT Studio (APlcexenicesetrl Studio) CNT DT. (CNT Development Team.)
</p>
