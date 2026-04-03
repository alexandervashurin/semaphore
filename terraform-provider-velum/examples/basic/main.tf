# Example: Basic Velum Project Setup

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

# Create a project
resource "velum_project" "main" {
  name        = "Infrastructure"
  description = "Infrastructure automation project"
}

# Create SSH access key
resource "velum_access_key" "ssh" {
  project_id  = velum_project.main.id
  name        = "Deploy SSH Key"
  type        = "ssh"
  private_key = file("~/.ssh/id_rsa")
  login_user  = "ansible"
}

# Create inventory
resource "velum_inventory" "main" {
  project_id = velum_project.main.id
  name       = "Production"
  type       = "static"
  inventory  = <<EOF
[webservers]
web1.example.com ansible_port=22
web2.example.com ansible_port=22

[dbservers]
db1.example.com ansible_port=22
EOF
}

# Create repository
resource "velum_repository" "main" {
  project_id = velum_project.main.id
  name       = "Ansible Playbooks"
  git_url    = "https://github.com/example/playbooks.git"
  git_branch = "main"
}

# Create environment
resource "velum_environment" "production" {
  project_id = velum_project.main.id
  name       = "Production"
  json       = jsonencode({
    ENVIRONMENT = "production"
    LOG_LEVEL   = "info"
    DB_HOST     = "db.example.com"
  })
}

# Create template
resource "velum_template" "deploy" {
  project_id     = velum_project.main.id
  name           = "Deploy Application"
  description    = "Deploy application to production servers"
  playbook       = "deploy.yml"
  inventory_id   = velum_inventory.main.id
  repository_id  = velum_repository.main.id
  environment_id = velum_environment.production.id
}

# Create schedule for daily backup
resource "velum_schedule" "backup" {
  project_id  = velum_project.main.id
  template_id = velum_template.deploy.id
  name        = "Daily Backup"
  cron        = "0 2 * * *"
  enabled     = true
}

# Outputs
output "project_id" {
  value = velum_project.main.id
}

output "template_id" {
  value = velum_template.deploy.id
}
