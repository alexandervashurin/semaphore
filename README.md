# Velum — Rust Edition

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org)
[![Build](https://github.com/tnl-o/velum/actions/workflows/rust.yml/badge.svg)](https://github.com/tnl-o/velum/actions)

A Rust rewrite of [Semaphore](https://github.com/semaphoreui/semaphore) — an open-source DevOps automation platform.
Manages and runs Ansible, Terraform, OpenTofu, Terragrunt, Bash, and PowerShell through a web UI backed by a PostgreSQL database.

> **Database:** PostgreSQL only (SQLite/MySQL removed in v2.2).
> **Tests:** 710 passing.

---

## Quick start

```bash
docker compose -f deploy/demo/docker-compose.yml up --build -d
```

Open **http://localhost:8088** · login `admin` / `admin123`

The demo image includes Ansible and seeds a sample project with an inventory, key, and playbook template.

---

## Running locally

**Prerequisites:** PostgreSQL running and accessible.

```bash
export SEMAPHORE_DB_DIALECT=postgres
export SEMAPHORE_DB_URL=postgres://semaphore:semaphore123@localhost:5432/semaphore

cd rust && cargo run -- server --host 0.0.0.0 --port 3000
```

**Create the first admin user:**

```bash
cd rust && cargo run -- user add \
  --username admin \
  --name "Administrator" \
  --email admin@localhost \
  --password admin123 \
  --admin
```

**Start PostgreSQL only (Docker):**

```bash
docker compose up postgres -d
```

---

## Deploy

Three ready-made stacks in `deploy/`:

| Stack | Directory | Purpose |
|---|---|---|
| Demo | `deploy/demo/` | One-command local demo, port 8088 |
| Dev | `deploy/dev/` | PostgreSQL + hot-reload for development |
| Prod | `deploy/prod/` | PostgreSQL + Nginx, production-ready |

Each directory contains `docker-compose.yml`, `.env.example`, and `README.md`.

---

## Configuration

All settings are environment variables.

| Variable | Description |
|---|---|
| `SEMAPHORE_DB_URL` | PostgreSQL connection string |
| `SEMAPHORE_WEB_PATH` | Path to frontend static files (default: `./web/public`) |
| `SEMAPHORE_ADMIN` | Username for the auto-created admin on first start |
| `SEMAPHORE_ADMIN_PASSWORD` | Password for the auto-created admin |
| `SEMAPHORE_ADMIN_EMAIL` | Email for the auto-created admin |
| `SEMAPHORE_ACCESS_KEY_ENCRYPTION` | Passphrase for AES-256-GCM encryption of stored keys |
| `RUST_LOG` | Log level: `debug`, `info`, `warn`, `error` (default: `info`) |

LDAP and OIDC options are documented in [`docs/technical/AUTH.md`](docs/technical/AUTH.md).
Full reference: [`docs/technical/CONFIG.md`](docs/technical/CONFIG.md).

---

## Features

### Core automation

- Run Ansible playbooks, Terraform/OpenTofu plans, Bash, PowerShell, Terragrunt
- Live log streaming over WebSocket during task execution
- Task history with full output stored per run
- **Dry Run mode** — validate without executing
- **Terraform Plan Preview** — show plan output before apply
- **Diff view** — side-by-side comparison between two task runs
- **Task Snapshots & Rollback** — one-click revert to a previous run state

### Project resources

- **Templates** — define what runs, with which inventory, keys, and environment; supports Views and Survey Forms
- **Inventories** — static YAML/INI, dynamic scripts, Terraform workspace, file-based
- **Repositories** — git checkout by branch, tag, or exact commit hash
- **Access Keys** — SSH keys, API tokens, login/password; encrypted at rest with AES-256-GCM
- **Environments** — key-value variables with secret masking
- **Schedules** — cron recurring runs and one-shot `run_at` datetime with auto-delete option
- **Webhooks** — incoming HTTP webhooks with integration matchers and aliases
- **Playbooks** — stored files with sync, run, and run history
- **Custom Credential Types** — AWX-style schema and injectors

### Workflow orchestration

- **Workflow Builder (DAG)** — define multi-step pipelines with dependency graph
- **Template Marketplace** — 11 community templates (Nginx, Docker, K8s, monitoring, …)
- **GitOps Drift Detection** — detect configuration drift between runs
- **Terraform Cost Tracking** — Infracost integration for cost estimates

### Team and access

- Multi-project architecture with per-project members
- Role-based access: Owner, Manager, Task Runner, Viewer
- Custom roles with bitmask permissions
- Member invites with accept links
- **LDAP Groups → Teams auto-sync**
- Audit log of all user actions

### Authentication

- Session login with bcrypt password hashing
- JWT tokens for API access
- TOTP two-factor authentication (RFC 6238, Google Authenticator / Authy compatible)
- TOTP recovery codes
- LDAP authentication with group sync
- OIDC / OAuth2 login

### Operations

- Backup and restore: full project export/import as JSON
- Secret Storages: HashiCorp Vault and DVLS integration
- Runners: self-registering agents with heartbeat and per-project tags
- Apps: configurable executors (Ansible, Terraform, Bash, Python, PowerShell, Pulumi, Terragrunt)
- Analytics dashboard (task counts, success rate, timeline)
- **Notification Policies** — Slack, Microsoft Teams, PagerDuty, generic webhook
- **AI Integration** — error analysis and playbook generation
- **Embedded MCP server** — 60 tools for AI-native DevOps control
- **Developer CLI** — `velum` binary for scripting and CI integration
- Prometheus metrics endpoint

---

## Tech stack

| | |
|---|---|
| **Runtime** | Rust stable, Tokio 1 |
| **Web framework** | Axum 0.8 (with WebSocket) |
| **Database** | SQLx 0.8, PostgreSQL |
| **Frontend** | Vanilla JS, Material Design, Roboto |
| **Auth** | JWT (jsonwebtoken 9), bcrypt, HMAC-SHA1 TOTP, ldap3, OIDC |
| **Encryption** | AES-256-GCM (aes-gcm 0.10) |
| **Scheduler** | cron 0.15 |
| **CI** | GitHub Actions — build, clippy, test |

---

## Development

```bash
cd rust
cargo check                      # compile check
cargo clippy -- -D warnings      # linter (0 warnings required)
cargo test                       # 710 tests
cargo run -- server              # start server
cargo run -- version             # print version
```

---

## Repository structure

```
├── rust/                   Backend — Rust / Axum / SQLx
│   └── src/
│       ├── api/            HTTP handlers and routing (197 handler functions)
│       ├── models/         Data models
│       ├── db/             Database layer (PostgreSQL)
│       ├── services/       Business logic (task runner, scheduler, backup, …)
│       └── config/         Configuration loading
├── web/public/             Frontend — 28 HTML pages, Vanilla JS
├── mcp/                    Embedded MCP server (Rust)
├── db/postgres/            PostgreSQL migration scripts
├── deploy/
│   ├── demo/               Demo stack (one command, port 8088)
│   ├── dev/                Development stack (hot-reload)
│   └── prod/               Production stack (Nginx, isolated networks)
├── docs/
│   ├── technical/          API, Auth, Config, Security, Webhooks, …
│   ├── guides/             Setup, Testing, Demo, Troubleshooting, …
│   ├── releases/           Release notes
│   ├── future/             Roadmap and planned features
│   └── archive/            Historical dev reports
├── scripts/                Utility scripts and SQL seeds
├── demo-playbooks/         Sample Ansible playbooks for the demo environment
├── Dockerfile              Production multi-stage image
└── docker-compose.yml      Full stack (PostgreSQL + backend)
```

---

## Documentation

| | |
|---|---|
| [docs/technical/API.md](docs/technical/API.md) | REST API reference |
| [docs/technical/AUTH.md](docs/technical/AUTH.md) | Authentication: JWT, TOTP, LDAP, OIDC |
| [docs/technical/CONFIG.md](docs/technical/CONFIG.md) | Environment variables |
| [docs/technical/BACKUP_RESTORE.md](docs/technical/BACKUP_RESTORE.md) | Backup and restore |
| [docs/guides/TROUBLESHOOTING.md](docs/guides/TROUBLESHOOTING.md) | Common issues |
| [docs/future/ROADMAP.md](docs/future/ROADMAP.md) | Roadmap |
| [CHANGELOG.md](CHANGELOG.md) | Version history |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Contribution guide |

---

## Related

| | |
|---|---|
| This repository | https://github.com/tnl-o/velum |
| Go original | [https://github.com/semaphoreui/semaphore](https://github.com/semaphoreui/semaphore) |
| Upstream fork | https://github.com/alexandervashurin/semaphore |

---

## License

MIT © [Alexander Vashurin](https://github.com/alexandervashurin)
