# Terraform Provider for Velum

Official Terraform provider for managing Velum resources.

## Requirements

- [Terraform](https://www.terraform.io/downloads.html) >= 1.0
- [Go](https://golang.org/doc/install) >= 1.21

## Building The Provider

1. Clone the repository
2. Build the provider

```bash
cd terraform-provider-velum
go build -o terraform-provider-velum
```

3. Install the provider locally

```bash
mkdir -p ~/.terraform.d/plugins/registry.terraform.io/tnl-o/velum/0.1.0/linux_amd64
cp terraform-provider-velum ~/.terraform.d/plugins/registry.terraform.io/tnl-o/velum/0.1.0/linux_amd64/
```

## Using the Provider

### Example Configuration

```hcl
terraform {
  required_providers {
    velum = {
      source  = "tnl-o/velum"
      version = "0.1.0"
    }
  }
}

provider "velum" {
  host    = "http://localhost:3000"
  api_key = var.velum_api_key
}

variable "velum_api_key" {
  description = "Velum API key"
  type        = string
  sensitive   = true
}
```

### Environment Variables

You can also configure the provider using environment variables:

```bash
export VELUM_HOST="http://localhost:3000"
export VELUM_API_KEY="your-api-key"
```

## Resources

### velum_project

Manage Velum projects.

```hcl
resource "velum_project" "main" {
  name        = "My Project"
  description = "Project description"
}
```

### velum_template

Manage task templates.

```hcl
resource "velum_template" "main" {
  project_id  = velum_project.main.id
  name        = "Deploy Application"
  description = "Deploy app to production"
  playbook    = "deploy.yml"
  inventory_id = velum_inventory.main.id
}
```

### velum_access_key

Manage access keys for SSH authentication.

```hcl
resource "velum_access_key" "ssh" {
  name       = "SSH Key"
  type       = "ssh"
  private_key = file("~/.ssh/id_rsa")
  login_user = "ansible"
}
```

```hcl
resource "velum_access_key" "password" {
  name           = "Login Password"
  type           = "login_password"
  login_password = "secret_password"
  login_user     = "admin"
}
```

### velum_inventory

Manage Ansible inventories.

```hcl
resource "velum_inventory" "main" {
  project_id = velum_project.main.id
  name       = "Production"
  type       = "static"
  inventory  = <<EOF
[webservers]
web1.example.com
web2.example.com

[dbservers]
db1.example.com
db2.example.com
EOF
}
```

### velum_repository

Manage Git repositories.

```hcl
resource "velum_repository" "main" {
  project_id = velum_project.main.id
  name       = "Ansible Playbooks"
  git_url    = "https://github.com/example/playbooks.git"
  git_branch = "main"
}
```

### velum_environment

Manage environment variables.

```hcl
resource "velum_environment" "main" {
  project_id = velum_project.main.id
  name       = "Production"
  json       = jsonencode({
    ENVIRONMENT = "production"
    LOG_LEVEL   = "info"
  })
}
```

### velum_schedule

Manage task schedules.

```hcl
resource "velum_schedule" "daily" {
  project_id   = velum_project.main.id
  template_id  = velum_template.main.id
  name         = "Daily Backup"
  cron         = "0 2 * * *"
  enabled      = true
}
```

## Data Sources

### velum_project

Read project information.

```hcl
data "velum_project" "main" {
  project_id = 1
}

output "project_name" {
  value = data.velum_project.main.name
}
```

### velum_template

Read template information.

```hcl
data "velum_template" "main" {
  project_id  = 1
  template_id = 42
}

output "template_playbook" {
  value = data.velum_template.main.playbook
}
```

### velum_inventory

Read inventory information.

```hcl
data "velum_inventory" "main" {
  project_id   = 1
  inventory_id = 10
}
```

## Developing the Provider

### Running Tests

```bash
go test ./...
```

### Building

```bash
go build -o terraform-provider-velum
```

### Releasing

To release a new version:

1. Update the version in `main.go`
2. Create a git tag
3. Build binaries for all platforms
4. Publish to Terraform Registry

## License

MIT License - see [LICENSE](../LICENSE) for details.
