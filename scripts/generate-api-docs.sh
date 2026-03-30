#!/bin/bash
# Скрипт для просмотра API документации

set -e

cd /home/a.vashurin@PSO.LOCAL/programms/github_my/semaphore

OUTPUT_DIR="./api-docs-html"

echo "╔════════════════════════════════════════════════════════╗"
echo "         Генерация HTML документации API                "
echo "╚════════════════════════════════════════════════════════╝"
echo ""

# Проверка наличия файлов
if [ -f "openapi.yml" ]; then
    SPEC_FILE="openapi.yml"
elif [ -f "api-docs.yml" ]; then
    SPEC_FILE="api-docs.yml"
else
    echo "❌ openapi.yml или api-docs.yml не найден"
    exit 1
fi

echo "📄 Найден $SPEC_FILE"

mkdir -p "$OUTPUT_DIR"

# Создаём HTML файл
cat > "$OUTPUT_DIR/index.html" << EOF
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Velum API Documentation</title>
    <link rel="icon" type="image/png" href="../web/public/logo.png">
    <style>
        body { margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; }
        .header { 
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white; 
            padding: 20px; 
            text-align: center;
        }
        .header h1 { margin: 0; }
        .header p { opacity: 0.9; }
        .links { padding: 20px; text-align: center; background: #f5f5f5; }
        .links a { margin: 0 10px; color: #667eea; text-decoration: none; }
    </style>
</head>
<body>
    <div class="header">
        <h1>🚀 Velum API Documentation</h1>
        <p>REST API для управления DevOps автоматизацией</p>
    </div>
    <div class="links">
        <a href="../docs/API.md">📄 Markdown версия</a>
        <a href="../openapi.yml">📋 OpenAPI Spec (YAML)</a>
        <a href="../api-docs.yml">📋 Swagger Spec (YAML)</a>
    </div>
    <redoc spec-url="../$SPEC_FILE"></redoc>
    <script src="https://cdn.jsdelivr.net/npm/redoc@latest/bundles/redoc.standalone.js"></script>
</body>
</html>
EOF

echo "✅ HTML документация создана в: $OUTPUT_DIR/index.html"
echo ""
echo "📖 Для просмотра откройте файл в браузере:"
echo "   file://$OUTPUT_DIR/index.html"
echo ""
echo "🌐 Или запустите локальный сервер:"
echo "   cd $OUTPUT_DIR && python3 -m http.server 8080"
echo "   Затем откройте: http://localhost:8080"
echo ""
echo "════════════════════════════════════════════════════════"
