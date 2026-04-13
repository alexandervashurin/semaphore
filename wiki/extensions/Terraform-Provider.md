# Terraform Provider

> Manage Velum resources via Terraform
>
> 📖 See also: [[VS Code Extension]], [[Configuration]], [[Docker Deployment]]

---

## Resources

| Resource | Description |
|----------|-------------|
| `velum_project` | Manage Velum projects |
| `velum_template` | Manage task templates |
| `velum_access_key` | Manage SSH/login access keys |

## Data Sources

| Data Source | Description |
|-------------|-------------|
| `velum_project` | Look up a project by name |
| `velum_template` | Look up a template by name |

---

## Example Usage

```hcl
terraform {
  required_providers {
    velum = {
      source  = "alexandervashurin/velum"
      version = "~> 0.1.0"
    }
  }
}

provider "velum" {
  server_url = "http://localhost:3000"
  api_token  = var.velum_api_token
}

# Look up existing project
data "velum_project" "main" {
  name = "My Project"
}

# Create a new template
resource "velum_template" "deploy" {
  project_id = data.velum_project.main.id
  name       = "Deploy to Production"
  playbook   = "deploy_prod.yml"
  type       = "ansible"
}

# Create an access key
resource "velum_access_key" "ssh" {
  project_id = data.velum_project.main.id
  name       = "Deploy Key"
  type       = "ssh"
  secret     = file("~/.ssh/id_rsa")
}
```

---

## Development

```bash
cd terraform-provider

# Build
go build -o terraform-provider-velum

# Test
go test ./...

# Package for local development
mkdir -p ~/.terraform.d/plugins/registry.terraform.io/alexandervashurin/velum/0.1.0/linux_amd64
cp terraform-provider-velum ~/.terraform.d/plugins/registry.terraform.io/alexandervashurin/velum/0.1.0/linux_amd64/
```

---

## Next Steps

- [[VS Code Extension]] — VS Code integration
- [[Configuration]] — environment variables
- [[Docker Deployment]] — Docker Compose options
