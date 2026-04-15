# VS Code Extension

> IntelliSense, snippets, and API integration for Velum
>
> 📖 See also: [[Terraform Provider]], [[Plugin System]], [[Development Setup]]

---

## Features

### Task Templates IntelliSense

Auto-complete for Ansible and Terraform files:
- Template names from your Velum server
- Ansible task keys (`name`, `hosts`, `become`, `tasks`, etc.)
- Ansible modules (`command`, `copy`, `service`, `k8s`, etc.)
- Terraform blocks (`resource`, `variable`, `provider`, etc.)

### Playbook Snippets

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

### Commands

| Command | Description |
|---------|-------------|
| `Velum: Login to Velum Server` | Configure server URL and API token |
| `Velum: List Projects` | Browse and select a project |
| `Velum: List Templates` | View templates for the selected project |
| `Velum: Run Task from Template` | Start a task from a template |
| `Velum: View Task Logs` | View output logs of a task |

---

## Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/alexandervashurin/semaphore.git
   ```

2. Open `vscode-extension/` in VS Code

3. Install dependencies:
   ```bash
   npm install
   ```

4. Press `F5` to launch the Extension Development Host

---

## Configuration

Open Settings (`Ctrl+,`) and search for `velum`:

| Setting | Default | Description |
|---------|---------|-------------|
| `velum.serverUrl` | `http://localhost:3000` | Velum server URL |
| `velum.apiToken` | *(empty)* | Your API token |
| `velum.projectId` | *(none)* | Default project ID for completion |

---

## Next Steps

- [[Terraform Provider]] — manage Velum via Terraform
- [[Plugin System]] — writing custom plugins
- [[Development Setup]] — local dev environment
