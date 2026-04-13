# Velum Wiki — Documentation Hub

> **Velum** — open-source DevOps automation platform (Rust + Kubernetes UI)
>
> 📦 [GitHub Repository](https://github.com/alexandervashurin/semaphore)
> 🐛 [Issue Tracker](https://github.com/alexandervashurin/semaphore/issues)
> 📋 [Changelog](../CHANGELOG.md)

---

## 📚 Documentation Sections

### 🚀 Getting Started
| Page | Description |
|------|-------------|
| [[Quick Start]] | Install and run Velum in 5 minutes |
| [[Configuration]] | Environment variables, config files, and defaults |
| [[First Project]] | Create your first project, template, and task |

### 🏗️ Deployment
| Page | Description |
|------|-------------|
| [[Docker Deployment]] | Docker Compose variants (demo, dev, prod) |
| [[Kubernetes Deployment]] | Deploy to K8s with Helm or manifests |
| [[Production Setup]] | Hardening, monitoring, and scaling |

### 🔌 API Reference
| Page | Description |
|------|-------------|
| [[REST API]] | Full REST API endpoint reference |
| [[GraphQL API]] | GraphQL schema and queries |
| [[WebSocket API]] | Real-time event streaming |
| [[MCP Server]] | Model Context Protocol tools (60 tools) |

### 🏛️ Architecture
| Page | Description |
|------|-------------|
| [[System Overview]] | High-level architecture diagram |
| [[Database Schema]] | PostgreSQL schema and migrations |
| [[Auth & Security]] | JWT, LDAP, OIDC, TOTP, RBAC |
| [[Task Execution Flow]] | How tasks are queued, run, and logged |
| [[Kubernetes Integration]] | K8s client, Helm, Jobs management |

### 🛠️ Development
| Page | Description |
|------|-------------|
| [[Development Setup]] | Local dev environment setup |
| [[Testing Guide]] | Writing and running tests |
| [[Code Structure]] | Rust module organization |

### 🧩 Extensions
| Page | Description |
|------|-------------|
| [[VS Code Extension]] | IntelliSense, snippets, API integration |
| [[Terraform Provider]] | Manage Velum resources via Terraform |
| [[Plugin System]] | Writing custom plugins |

### 🐛 Troubleshooting
| Page | Description |
|------|-------------|
| [[Common Issues]] | FAQ and known problems |
| [[Debug Mode]] | Enabling verbose logging |
| [[Migration Guide]] | Upgrading between versions |

### 🤝 Contributing
| Page | Description |
|------|-------------|
| [[Contributing Guide]] | How to contribute code and docs |
| [[Code of Conduct]] | Community guidelines |
| [[Security Policy]] | Reporting vulnerabilities |

---

## 📊 Project Stats

| Metric | Value |
|--------|-------|
| **Language** | Rust (backend) + Vanilla JS (frontend) |
| **Tests** | 6551 unit tests, ~85% coverage |
| **API Endpoints** | 135+ REST + GraphQL + WebSocket |
| **Docker Size** | ~23MB (optimized scratch build) |
| **Platforms** | Linux amd64/arm64, macOS amd64/arm64 |

---

## 🔗 External Links

- [OpenAPI Specification](../api-docs.yml)
- [Docker Compose Files](../docker-compose.yml)
- [Postman Collection](../.postman/)
- [Demo Data Scripts](../scripts/)
