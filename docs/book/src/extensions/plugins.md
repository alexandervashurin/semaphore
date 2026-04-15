# Система плагинов

> Написание своих плагинов для Velum
>
> 📖 См. также: [VS Code](./vscode.md), [Terraform](./terraform.md), [MCP сервер](../api-reference/mcp-server.md)

---

## Обзор

Velum поддерживает систему плагинов на базе WASM (WebAssembly), позволяющую расширять функциональность без изменения основного кода.

## Поддерживаемые точки расширения

- **Хуки задач** — выполнение кода до/после задач
- **Пользовательские обработчики** — своя логика при событиях
- **Интеграции** — подключение внешних систем

## WASM Runtime

Плагины компилируются в WASM и загружаются в безопасную песочницу.

```bash
# Компиляция плагина
cargo build --target wasm32-unknown-unknown --release

# Размещение плагина
cp target/wasm32-unknown-unknown/release/my_plugin.wasm /velum/plugins/
```

---

## Следующие шаги

- [VS Code](./vscode.md) — расширение VS Code
- [Terraform](./terraform.md) — управление через Terraform
- [MCP сервер](../api-reference/mcp-server.md) — AI-интеграция
