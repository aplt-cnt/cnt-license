# clicense

A CLI tool for generating open source license files, written in Rust.

**clicense** helps you quickly create license files for your projects. It ships with 10 popular open source license templates built-in, supports custom licenses, online updates with diff, and a configurable default system that lets you generate a LICENSE file with a single command.

## Features

- **10 built-in license templates** — MIT, Apache-2.0, GPL-3.0, LGPL-3.0, BSD-3-Clause, BSD-2-Clause, MPL-2.0, Unlicense, ISC, EPL-2.0
- **Custom licenses** — Add and manage your own license templates
- **Configurable defaults** — Set default author, year, and license so `clicense new` just works
- **Online updates with diff** — Pull new/updated templates from a remote server, see what changed before writing
- **Remote browsing** — List, inspect, and download licenses directly from a remote server
- **API server included** — Self-host your own license template service with the companion `cnt-license-server`

## Installation

### From source

```bash
git clone https://github.com/user/cnt-license.git
cd cnt-license
cargo install --path .
```

This installs the `clicense` binary. To also install the server:

```bash
cargo install --path server
```

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

# View detailed info about a license
clicense list gpl-3.0

# Output license text to stdout (with placeholder substitution)
clicense source mit -a "Your Name" -y 2026
```

## CLI Reference

### `clicense new [license_id]`

Generate a license file.

| Option | Short | Description |
|--------|-------|-------------|
| `license_id` | — | License identifier (e.g. `mit`, `apache-2.0`). Falls back to `default_license` config |
| `--author` | `-a` | Copyright holder. Falls back to `default_author` config |
| `--year` | `-y` | Copyright year. Falls back to `default_year` config, then current year |
| `--output` | `-o` | Output file name. Falls back to `output_name` config, then `LICENSE` |

**Parameter priority**: CLI argument > config default > fallback value.

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

Config is stored at `~/.clicense/config.yml`.

### `clicense update [--update-url URL]`

Download new/updated license templates from a remote server. Shows a diff summary before writing:

- 🟢 **Added** — New on server, not locally present
- 🟡 **Updated** — Content differs from local copy
- ⚪ **Unchanged** — Identical, skipped
- 🔴 **Local only** — Only exists locally, preserved

Use `--update-url` to temporarily override the server URL without modifying config.

### `clicense add <file> --name <id> [--force]`

Add a custom license template from a file. Use `--force` to overwrite an existing one.

### `clicense remove <names...> [--all]`

Remove custom license templates. Pass multiple names to batch-remove, or `--all` to remove every custom license.

### `clicense list [name] [--builtin|--custom]`

List installed licenses. Without a name, shows all licenses in a table. With a name, shows detailed three-column info (Permissions / Conditions / Limitations).

| Flag | Description |
|------|-------------|
| `--builtin` | Show only built-in licenses |
| `--custom` | Show only custom licenses |

### `clicense online [--online-url URL] <subcommand>`

Interact with a remote license server.

| Subcommand | Description |
|------------|-------------|
| `list` | List all licenses available on the server |
| `license <name>` | Show detailed info for a remote license |
| `source <name>` | Output raw license text from the server |

### `clicense source <name> [-a author] [-y year]`

Output the raw content of a local license to stdout. Supports `{year}` and `{author}` placeholder substitution.

### `clicense version`

Show the current version.

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

## API Server

The project includes `cnt-license-server`, a lightweight REST API server built with Axum for hosting license templates.

### Start the server

```bash
# From the workspace root
cargo run -p cnt-license-server

# Custom port and licenses directory
PORT=8080 LICENSES_DIR=./licenses cargo run -p cnt-license-server
```

Default: `0.0.0.0:3000`.

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/v1/health` | Health check |
| `GET` | `/api/v1/version` | Server version |
| `GET` | `/api/v1/licenses` | List all license templates |
| `GET` | `/api/v1/licenses/{id}` | Get a single license template |
| `GET` | `/api/v1/licenses/{id}/info` | Get license metadata |
| `GET` | `/api/v1/search?q=` | Fuzzy search licenses |

Backward-compatible routes (`/health`, `/licenses`, `/search`, etc.) are also available.

### Content negotiation

- Default response format: **YAML** (compatible with `clicense update`)
- With `Accept: application/json` header: **JSON**

## Project Structure

```
cnt-license/                    Cargo workspace root
├── Cargo.toml                 Workspace config + CLI package
├── licenses/                  10 built-in license templates (*.txt)
├── licenses-meta.toml         License metadata (permissions, conditions, limitations)
├── src/                       CLI source code
│   ├── main.rs                Entry point, clap command definitions
│   ├── config.rs              Configuration management (~/.clicense/config.yml)
│   ├── license.rs             Built-in license templates (LazyLock + include_str!)
│   ├── metadata.rs            License metadata loader
│   ├── http.rs                Shared HTTP client (ureq agent)
│   └── command/               Command implementations
│       ├── mod.rs
│       ├── new.rs             Generate license files
│       ├── config_cmd.rs      Config management
│       ├── update.rs          Online update with diff
│       ├── add.rs             Add custom licenses
│       ├── remove.rs          Remove custom licenses
│       ├── list.rs            List/view licenses
│       ├── online.rs          Remote server interaction
│       └── source.rs          Output raw license text
└── server/                    API server sub-project
    ├── Cargo.toml
    └── src/
        ├── main.rs            Axum routes + startup
        ├── config.rs          Environment variable config
        ├── state.rs           Shared application state
        ├── data/
        │   ├── mod.rs         License data loader
        │   └── licenses.toml  License metadata (server-side)
        ├── handlers/          Route handlers
        │   ├── health.rs
        │   ├── version.rs
        │   ├── licenses.rs
        │   └── search.rs
        └── models/
            └── license.rs     Data models (LicenseMeta, SearchResponse, ...)
```

## Tech Stack

**CLI**:
- [Rust](https://www.rust-lang.org/) (edition 2024)
- [clap](https://docs.rs/clap) 4.6 — command-line argument parser (derive)
- [ureq](https://docs.rs/ureq) 3 — HTTP client
- [serde](https://docs.rs/serde) + [serde_yaml](https://docs.rs/serde_yaml) + [toml](https://docs.rs/toml) — serialization
- [colored](https://docs.rs/colored) — terminal colors
- [dirs](https://docs.rs/dirs) 6 — platform-specific directories

**Server**:
- [Axum](https://docs.rs/axum) 0.8 — web framework
- [Tokio](https://docs.rs/tokio) 1 — async runtime
- [tower-http](https://docs.rs/tower-http) — CORS + request tracing
- [tracing](https://docs.rs/tracing) — structured logging

## License

This project is licensed under the [GNU General Public License v3.0](LICENSE).

---

<p align="center">
  <strong>clicense</strong> by APLT Studio (APlcexenicesetrl Studio) CNT DT. (CNT Development Team.)
</p>
