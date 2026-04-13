# Velum — VS Code Extension

Integrates Visual Studio Code with [Velum](https://github.com/alexandervashurin/semaphore) — the open-source DevOps automation platform.

## Features

### Task Templates IntelliSense
- Auto-complete for Ansible playbook keys (`name`, `hosts`, `become`, `tasks`, etc.)
- Auto-complete for Ansible modules (`command`, `copy`, `service`, `k8s`, etc.)
- Template names from your Velum server appear as completion suggestions
- Terraform block completion (`resource`, `variable`, `provider`, etc.)

### Playbook Snippets
Type a prefix and press `Tab` to insert:

| Prefix | Description |
|--------|-------------|
| `velum-playbook` | Full Ansible playbook scaffold |
| `velum-task` | Single Ansible task |
| `velum-role` | Include Ansible role |
| `velum-handler` | Handler definition |
| `velum-tf-provider` | Terraform provider block |
| `velum-tf-resource` | Terraform resource block |
| `velum-tf-variable` | Terraform variable with validation |
| `velum-params` | Velum task parameters docs |
| `velum-config` | Velum run configuration JSON |

### Commands
| Command | Description |
|---------|-------------|
| `Velum: Login to Velum Server` | Configure server URL and API token |
| `Velum: List Projects` | Browse and select a project |
| `Velum: List Templates` | View templates for the selected project |
| `Velum: Run Task from Template` | Start a task from a template |
| `Velum: View Task Logs` | View output logs of a task |

### Configuration
Open Settings (`Ctrl+,`) and search for `velum`:

| Setting | Default | Description |
|---------|---------|-------------|
| `velum.serverUrl` | `http://localhost:3000` | Velum server URL |
| `velum.apiToken` | *(empty)* | Your API token |
| `velum.projectId` | *(none)* | Default project ID for completion |

## Installation

1. Clone the repository and open `vscode-extension/` in VS Code
2. Run `npm install`
3. Press `F5` to launch the Extension Development Host
4. Or package: `vsce package` → install the `.vsix`

## Requirements

- VS Code 1.85+
- Velum server running and accessible
- API token (obtain from Velum UI → Settings → API Keys)

## License

MIT — see [LICENSE](../LICENSE)
