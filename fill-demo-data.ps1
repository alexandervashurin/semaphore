# ============================================================================
# Наполнение БД тестовыми данными для Semaphore UI (Rust)
# PowerShell версия
# ============================================================================

$API_URL = "http://localhost:8088/api"
$USERNAME = "admin"
$PASSWORD = "admin123"

function Get-Token {
    Write-Host "Авторизация..." -ForegroundColor Cyan
    $body = @{
        username = $USERNAME
        password = $PASSWORD
    } | ConvertTo-Json
    
    try {
        $resp = Invoke-RestMethod -Uri "$API_URL/auth/login" -Method Post -ContentType "application/json" -Body $body
        Write-Host "Токен получен" -ForegroundColor Green
        return $resp.token
    } catch {
        Write-Host "Ошибка авторизации: $_" -ForegroundColor Red
        exit 1
    }
}

function Invoke-Post {
    param($Endpoint, $Data)
    try {
        $headers = @{
            "Authorization" = "Bearer $TOKEN"
            "Content-Type" = "application/json"
        }
        $resp = Invoke-RestMethod -Uri "$API_URL$Endpoint" -Method Post -Headers $headers -Body ($Data | ConvertTo-Json -Compress)
        return $resp
    } catch {
        Write-Host "Ошибка POST $Endpoint : $_" -ForegroundColor Yellow
        return $null
    }
}

function New-Project {
    param($Name)
    Write-Host "Создание проекта: $Name" -ForegroundColor Cyan
    $data = @{
        name = $Name
        alert = $false
        max_parallel_tasks = 0
    }
    $resp = Invoke-Post "/projects" $data
    if ($resp -and $resp.id) {
        Write-Host "Проект '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function New-Key {
    param($ProjectId, $Name, $Type, $Login = $null, $Secret = $null)
    Write-Host "Создание ключа: $Name (type=$Type)" -ForegroundColor Cyan
    $data = @{
        name = $Name
        type = $Type
    }
    if ($Login) { $data.login = $Login }
    if ($Secret) { $data.secret = $Secret }
    $resp = Invoke-Post "/project/$ProjectId/keys" $data
    if ($resp -and $resp.id) {
        Write-Host "Ключ '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function New-Repo {
    param($ProjectId, $Name, $Url, $Branch = "main")
    Write-Host "Создание репозитория: $Name" -ForegroundColor Cyan
    $data = @{
        name = $Name
        git_url = $Url
        git_branch = $Branch
    }
    $resp = Invoke-Post "/project/$ProjectId/repositories" $data
    if ($resp -and $resp.id) {
        Write-Host "Репозиторий '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function New-Inventory {
    param($ProjectId, $Name, $Type, $Data)
    Write-Host "Создание инвентаря: $Name (type=$Type)" -ForegroundColor Cyan
    $data = @{
        name = $Name
        inventory_type = $Type
        inventory = $Data
    }
    $resp = Invoke-Post "/project/$ProjectId/inventory" $data
    if ($resp -and $resp.id) {
        Write-Host "Инвентарь '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function New-Environment {
    param($ProjectId, $Name, $JsonVars)
    Write-Host "Создание окружения: $Name" -ForegroundColor Cyan
    $data = @{
        name = $Name
        json = $JsonVars
    }
    $resp = Invoke-Post "/project/$ProjectId/environment" $data
    if ($resp -and $resp.id) {
        Write-Host "Окружение '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function New-Template {
    param($ProjectId, $Name, $Playbook, $InventoryId, $RepositoryId = 0, $EnvironmentId = 0, $App = "ansible")
    Write-Host "Создание шаблона: $Name" -ForegroundColor Cyan
    $data = @{
        name = $Name
        playbook = $Playbook
        inventory_id = $InventoryId
        app = $App
    }
    if ($RepositoryId -gt 0) { $data.repository_id = $RepositoryId }
    if ($EnvironmentId -gt 0) { $data.environment_id = $EnvironmentId }
    $resp = Invoke-Post "/project/$ProjectId/templates" $data
    if ($resp -and $resp.id) {
        Write-Host "Шаблон '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function New-Schedule {
    param($ProjectId, $Name, $TemplateId, $Cron)
    Write-Host "Создание расписания: $Name ($Cron)" -ForegroundColor Cyan
    $data = @{
        id = 0
        name = $Name
        template_id = $TemplateId
        cron = $Cron
        active = $true
        project_id = $ProjectId
    }
    $resp = Invoke-Post "/project/$ProjectId/schedules" $data
    if ($resp -and $resp.id) {
        Write-Host "Расписание '$Name' (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

function Start-Task {
    param($ProjectId, $TemplateId)
    Write-Host "Запуск задачи (шаблон #$TemplateId)..." -ForegroundColor Cyan
    $data = @{
        template_id = $TemplateId
    }
    $resp = Invoke-Post "/project/$ProjectId/tasks" $data
    if ($resp -and $resp.id) {
        Write-Host "Задача запущена (ID: $($resp.id))" -ForegroundColor Green
        return $resp.id
    }
    return 0
}

# ════════════════════════════════════════════════════════════════════════════
Write-Host ""
Write-Host "API: $API_URL" -ForegroundColor Blue
Write-Host ""

$TOKEN = Get-Token

# ── Проект 1: Demo Project ──────────────────────────────────────────────────
Write-Host ""
Write-Host "== Проект 1: Demo Project ==" -ForegroundColor Yellow
$P1 = New-Project "Demo Project"

New-Key $P1 "No Key (none)" "none" "" ""
New-Key $P1 "demo user (password)" "login_password" "demo" "demo123"

$R1 = New-Repo $P1 "Demo Playbooks (local)" "file:///app/playbooks" "main"

$I_LOCAL = New-Inventory $P1 "Localhost" "static" @"
[local]
localhost ansible_connection=local
"@

$I_TARGET = New-Inventory $P1 "Demo Target (ansible-target)" "static" @"
[demo_servers]
ansible-target ansible_user=demo ansible_password=demo123 ansible_ssh_common_args='-o StrictHostKeyChecking=no'
"@

$E_DEV = New-Environment $P1 "Development" '{"ENV": "development", "DEBUG": "true", "APP_VERSION": "1.0.0"}'
$E_PROD = New-Environment $P1 "Production" '{"ENV": "production", "DEBUG": "false", "APP_VERSION": "2.5.1"}'

$T_HELLO = New-Template $P1 "Hello World (localhost)" "hello.yml" $I_LOCAL $R1 $E_DEV "ansible"
$T_PING = New-Template $P1 "Ping Demo Servers" "ping.yml" $I_TARGET $R1 $E_DEV "ansible"
$T_DEPLOY = New-Template $P1 "Deploy Web App" "deploy-web.yml" $I_TARGET $R1 $E_PROD "ansible"

New-Schedule $P1 "Hourly Hello" $T_HELLO "0 * * * *"
New-Schedule $P1 "Nightly Deploy" $T_DEPLOY "0 2 * * *"
Start-Task $P1 $T_HELLO

# ── Проект 2: Infrastructure ────────────────────────────────────────────────
Write-Host ""
Write-Host "== Проект 2: Infrastructure ==" -ForegroundColor Yellow
$P2 = New-Project "Infrastructure"

New-Key $P2 "Deploy SSH Key" "ssh" "ubuntu" ""
New-Key $P2 "AWS Access" "login_password" "aws_key_id" "aws_secret_key"
New-Repo $P2 "Infrastructure Code" "https://github.com/example/infra.git" "main"
New-Repo $P2 "Terraform Modules" "https://github.com/example/terraform-modules.git" "main"

$I_PROD = New-Inventory $P2 "Production Servers" "static" @"
[production]
prod1.example.com
prod2.example.com
"@

New-Inventory $P2 "Staging Servers" "static" @"
[staging]
staging1.example.com
"@

New-Environment $P2 "AWS us-east-1" '{"AWS_REGION": "us-east-1", "TF_VAR_env": "prod"}'
New-Environment $P2 "GCP europe-west" '{"GOOGLE_PROJECT": "my-project"}'
$T_PROV = New-Template $P2 "Provision Servers" "provision.yml" $I_PROD "" "" "ansible"
New-Schedule $P2 "Weekly Provision" $T_PROV "0 3 * * 1"

# ── Проект 3: Web Applications ───────────────────────────────────────────────
Write-Host ""
Write-Host "== Проект 3: Web Applications ==" -ForegroundColor Yellow
$P3 = New-Project "Web Applications"

New-Key $P3 "Deploy User" "login_password" "deploy" "deploy123"
New-Repo $P3 "Web App Repo" "https://github.com/example/webapp.git" "main"

$I_WEB = New-Inventory $P3 "Web Servers" "static" @"
[webservers]
web1.example.com
web2.example.com

[dbservers]
db1.example.com
"@

New-Environment $P3 "Production" '{"APP_ENV": "production", "NODE_ENV": "production"}'

$T_FE = New-Template $P3 "Deploy Frontend" "deploy.yml" $I_WEB "" "" "ansible"
$T_SSL = New-Template $P3 "Update SSL Certs" "ssl-renew.yml" $I_WEB "" "" "ansible"
New-Template $P3 "Restart Services" "restart.yml" $I_WEB "" "" "ansible"
New-Schedule $P3 "Nightly Deploy" $T_FE "0 1 * * *"
New-Schedule $P3 "Monthly SSL Renew" $T_SSL "0 4 1 * *"

# ── Итог ─────────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "+----------------------------------------------------------+" -ForegroundColor Green
Write-Host "|   Тестовые данные созданы!                               |" -ForegroundColor Green
Write-Host "+----------------------------------------------------------+" -ForegroundColor Green
Write-Host ""
Write-Host "  Проектов: 3 | Шаблонов: 8 | Расписаний: 5" -ForegroundColor White
Write-Host ""
Write-Host "URL: http://localhost:8088 (admin / admin123)" -ForegroundColor Cyan
Write-Host ""
Write-Host "Реальный ansible:" -ForegroundColor Yellow
Write-Host "  Demo Project -> Шаблоны -> 'Hello World (localhost)' -> Run" -ForegroundColor White
Write-Host "  Demo Project -> Шаблоны -> 'Ping Demo Servers' -> Run" -ForegroundColor White
