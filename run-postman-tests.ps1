# ============================================================================
# Velum — Postman Collection Runner (PowerShell)
# Запуск всех API тестов из Postman коллекции
# ============================================================================

$ErrorActionPreference = "Stop"

# Конфигурация
$BASE_URL = if ($env:BASE_URL) { $env:BASE_URL } else { "http://localhost:8088" }
$COLLECTION_FILE = ".postman\postman\collections\Semaphore API.json"
$ENV_FILE = ".postman\environments\velum-demo.postman_environment.json"
$REPORT_FILE = "newman-report.json"

Write-Host ""
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host "              🚀 Velum — Postman Collection Runner" -ForegroundColor Cyan
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host ""

# Проверка коллекции
Write-Host "ℹ️  Проверка коллекции..." -ForegroundColor Blue

if (-not (Test-Path $COLLECTION_FILE)) {
    Write-Host "❌ Коллекция не найдена: $COLLECTION_FILE" -ForegroundColor Red
    exit 1
}

Write-Host "✅ Коллекция найдена: $COLLECTION_FILE" -ForegroundColor Green

# Проверка доступности API
Write-Host ""
Write-Host "ℹ️  Проверка доступности API..." -ForegroundColor Blue

try {
    $response = Invoke-RestMethod -Uri "$BASE_URL/api/health" -TimeoutSec 5 -ErrorAction Stop
    Write-Host "✅ API доступно: $BASE_URL" -ForegroundColor Green
} catch {
    Write-Host "⚠️  API недоступно: $BASE_URL" -ForegroundColor Yellow
    Write-Host "Убедитесь, что сервер запущен:" -ForegroundColor Yellow
    Write-Host "  docker compose -f docker-compose.demo.yml up" -ForegroundColor Yellow
    exit 1
}

# Создание environment файла
Write-Host ""
Write-Host "ℹ️  Создание environment конфигурации..." -ForegroundColor Blue

$envDir = ".postman\environments"
if (-not (Test-Path $envDir)) {
    New-Item -ItemType Directory -Path $envDir | Out-Null
}

$envConfig = @{
    id = "velum-demo-env"
    name = "Velum Demo"
    values = @(
        @{
            key = "baseUrl"
            value = "$BASE_URL/api"
            type = "default"
            enabled = $true
        },
        @{
            key = "username"
            value = "admin"
            type = "default"
            enabled = $true
        },
        @{
            key = "password"
            value = "admin123"
            type = "default"
            enabled = $true
        }
    )
    "_postman_variable_scope" = "environment"
}

$envConfig | ConvertTo-Json -Depth 10 | Out-File -FilePath $ENV_FILE -Encoding utf8

Write-Host "✅ Environment создан: $ENV_FILE" -ForegroundColor Green

Write-Host ""
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host "                         Запуск тестов" -ForegroundColor Cyan
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host ""

# Запуск Newman
Write-Host "Запуск newman run..." -ForegroundColor Gray
newman run $COLLECTION_FILE --environment $ENV_FILE --reporters "cli,json" --reporter-json-export $REPORT_FILE --delay-request 100 --timeout 30000 --ignore-redirects

Write-Host ""
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host "                           Результаты" -ForegroundColor Cyan
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host ""

# Проверка результатов
if (Test-Path $REPORT_FILE) {
    Write-Host "✅ Отчёт сохранён: $REPORT_FILE" -ForegroundColor Green
    
    # Парсинг результатов (если есть jq)
    if (Get-Command jq -ErrorAction SilentlyContinue) {
        $report = Get-Content $REPORT_FILE -Raw | ConvertFrom-Json
        $total = $report.run.stats.total
        $failed = $report.run.failures.Count
        $passed = $total - $failed
        
        Write-Host ""
        Write-Host "📊 Статистика:" -ForegroundColor Yellow
        Write-Host "   Всего тестов: $total"
        Write-Host "   ✅ Успешно: $passed"
        Write-Host "   ❌ Ошибок: $failed"
        
        if ($failed -gt 0) {
            Write-Host ""
            Write-Host "🔴 Ошибки:" -ForegroundColor Red
            foreach ($failure in $report.run.failures) {
                Write-Host "   - $($failure.error.name): $($failure.error.message)" -ForegroundColor Red
            }
        }
    } else {
        Write-Host ""
        Write-Host "ℹ️  Установите jq для детальной статистики:" -ForegroundColor Yellow
        Write-Host "   choco install jq" -ForegroundColor Yellow
    }
} else {
    Write-Host "⚠️  Отчёт не создан" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "============================================================================" -ForegroundColor Cyan
Write-Host ""
