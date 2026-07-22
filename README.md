# opencodeR

**Rust port of the OpenCode AI coding agent.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

opencodeR is a full-featured HTTP API server and CLI client for AI-assisted coding, rewritten from the original [opencode](https://github.com/anomalyco/opencode) TypeScript codebase into Rust. It provides the same API surface with better performance, lower memory usage, and cross-platform binaries.

## Quick Start

```bash
# Download the latest release for your platform
curl -fsSL https://github.com/grave0x/opencodeR/releases/latest/download/opencodeR-x86_64-unknown-linux-gnu.tar.gz | tar xz

# Start the server
./opencodeR-server --headless --port 8081

# In another terminal, run a prompt
./opencodeR run "Explain this codebase" --port 8081
```

## Binaries

| Binary | Description | Default mode |
|--------|-------------|-------------|
| `opencodeR` | Combined binary — server + client + TUI | Classic interactive REPL |
| `opencodeR-server` | HTTP API server | Server dashboard TUI (`--headless` for headless) |
| `opencodeR-client` | Remote CLI client | Connects to running server (`--base-url`) |

### opencodeR (combined)

```
USAGE:
    opencodeR [COMMAND]

COMMANDS:
    server      Start the headless HTTP server
    client      Connect to a remote server interactively
    run         Run a one-shot prompt and exit
    tui         Launch the server dashboard TUI
    sessions    List sessions from a running server
    models      List available AI models
    providers   List configured providers
    agents      List available agents
    export      Export a session as JSON
    import      Import session data from JSON file
    completion  Generate shell completion script
```

### opencodeR-server

```
# Server dashboard TUI (default)
opencodeR-server

# Headless mode
opencodeR-server --headless --port 8081 --password mysecret
```

### opencodeR-client

```
# Interactive REPL
opencodeR-client --base-url http://127.0.0.1:8081

# One-shot prompt
opencodeR-client --base-url http://127.0.0.1:8081 --one-shot "Refactor this"
```

## API Endpoints

The server implements the full opencode HTTP API:

| Group | Endpoints | Status |
|-------|-----------|--------|
| **Health** | `GET /api/health` | ✅ |
| **Sessions** | CRUD + prompt, events, history, messages, revert, interrupt, context, wait | ✅ |
| **Agents** | `GET /api/agent` | ✅ |
| **Models** | `GET /api/model` | ✅ |
| **Providers** | `GET /api/provider/:id` | ✅ |
| **Filesystem** | `GET /api/fs/read/*path`, `list`, `find` | ✅ |
| **PTY** | CRUD + connect token, WebSocket connect | ✅ (real processes) |
| **Events** | `GET /api/event` (SSE) | ✅ |
| **Integrations** | List, connect key/OAuth, attempt flow | ✅ |
| **Permissions** | Request/reply flow, saved rules | ✅ |
| **Questions** | Ask/reply/reject | ✅ |
| **Commands** | `GET /api/command` | ✅ |
| **Skills** | `GET /api/skill` | ✅ |
| **Auth** | Password-based Basic auth middleware | ✅ |

## Architecture

```
                   ┌─────────────────────────┐
                   │   opencodeR binary       │
                   │  (CLI / TUI / Server)    │
                   └─────────┬───────────────┘
                             │
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
      ┌──────────┐   ┌──────────┐   ┌──────────┐
      │  Server  │   │  Client  │   │   TUI    │
      │  Crate   │   │  Crate   │   │  (ratatui)│
      └────┬─────┘   └────┬─────┘   └──────────┘
           │              │
           └──────┬───────┘
                  ▼
          ┌──────────────┐
          │  Core Crate  │
          │ (traits +    │
          │  in-memory)  │
          └──────┬───────┘
                 │
          ┌──────┴───────┐
          │  Schema +    │
          │  Protocol    │
          │  Crates      │
          └──────────────┘
```

### Crate structure

| Crate | Purpose |
|-------|---------|
| `opencode-r-schema` | Data types (session, agent, model, provider, PTY, etc.) |
| `opencode-r-protocol` | HTTP routes, error types, request/response payloads |
| `opencode-r-core` | Service traits + in-memory implementations |
| `opencode-r-server` | Axum router, handlers, middleware (auth, access log) |
| `opencode-r-client` | HTTP client library for the API |
| `opencode-r-cli` | Binary entry points + TUI |
| `opencode-r-llm` | LLM provider abstraction (in progress) |
| `opencode-r-plugin` | Plugin system (in progress) |

## Building from source

```bash
git clone https://github.com/grave0x/opencodeR.git
cd opencodeR
cargo build --release --bin opencodeR --bin opencodeR-server --bin opencodeR-client
```

The release binaries will be in `target/release/`.

### Cross-compilation

```bash
# Linux ARM64
cargo build --release --target aarch64-unknown-linux-gnu --bin opencodeR

# Windows
cargo build --release --target x86_64-pc-windows-msvc --bin opencodeR
```

## Packaging

| Format | Location |
|--------|----------|
| **Arch Linux** | `packaging/arch/PKGBUILD` |
| **Debian/Ubuntu** | `packaging/debian/` |
| **RPM (Fedora/RHEL)** | `packaging/rpm/opencodeR.spec` |
| **Cargo** | `cargo install opencode-r-cli` |

## Configuration

| Environment variable | Default | Description |
|---------------------|---------|-------------|
| `OPENCODE_PORT` | `8081` | Server port |
| `OPENCODE_PASSWORD` | (none) | Enable Basic auth |
| `OPENCODE_BASE_URL` | `http://127.0.0.1:8081` | Server URL for client |
| `RUST_LOG` | `info` | Log level (e.g. `debug`, `opencode_r_server=info`) |

## Feature comparison

See [`features/feature-comparison.json`](features/feature-comparison.json) for a detailed
comparison between opencodeR and the original opencode.

See [`features/ROADMAP.md`](features/ROADMAP.md) for the planned feature roadmap.

## License

MIT — see [LICENSE](LICENSE).
