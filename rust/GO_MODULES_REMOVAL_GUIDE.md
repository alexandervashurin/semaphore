# Руководство по удалению Go модулей

**Дата**: 2026-02-26  
**Статус**: ✅ ГОТОВО К ВЫПОЛНЕНИЮ

---

## 📋 Обзор

Это руководство описывает процесс удаления Go модулей, которые были полностью переписаны на Rust.

### Модули для удаления

| Модуль | Rust аналог | Статус | Файлов используют |
|--------|-------------|--------|-------------------|
| `pkg/task_logger` | `services/task_logger.rs` | ✅ Готов | 30+ |
| `pkg/ssh` | `services/ssh_agent.rs` | ✅ Готов | 7 |

---

## ⚠️ ВАЖНО: Перед удалением

### 1. Убедитесь, что Rust код работает

```bash
cd rust

# Проверка компиляции
cargo build --release

# Запуск тестов
cargo test

# Результат: 125 тестов должны пройти
```

### 2. Сделайте резервную копию

```bash
# Создайте backup Go модулей
cd ..
tar -czf pkg_backup_$(date +%Y%m%d).tar.gz pkg/
```

### 3. Проверьте зависимости

```bash
# Найдите все использования pkg/task_logger
grep -r "pkg/task_logger" --include="*.go" .

# Найдите все использования pkg/ssh
grep -r "\"github.com/semaphoreui/semaphore/pkg/ssh\"" --include="*.go" .
```

---

## 🗑 Этап 1: Удаление pkg/task_logger

### Шаг 1.1: Анализ зависимостей

```bash
# Найти все файлы, использующие pkg/task_logger
grep -r "pkg/task_logger" --include="*.go" . | cut -d: -f1 | sort -u
```

**Ожидаемый результат** (30+ файлов):
```
./api/runners/runners.go
./api/tasks/tasks.go
./db/Task.go
./db/Store.go
./db_lib/AccessKeyInstaller.go
./db_lib/AnsibleApp.go
./db_lib/GoGitClient.go
./db_lib/TerraformApp.go
./services/tasks/TaskPool.go
./services/tasks/TaskRunner.go
...
```

### Шаг 1.2: Подготовка замены

**ВАРИАНТ A: Использование Rust через FFI**

Создайте Go обёртку для Rust FFI:

```go
// pkg/rustlib/task_logger.go
package rustlib

/*
#cgo LDFLAGS: -L${SRCDIR}/lib -lsemaphore_ffi
#include <stdlib.h>
#include <stdint.h>

typedef enum {
    C_TaskStatus_Waiting = 0,
    C_TaskStatus_Starting = 1,
    C_TaskStatus_WaitingConfirmation = 2,
    C_TaskStatus_Confirmed = 3,
    C_TaskStatus_Rejected = 4,
    C_TaskStatus_Running = 5,
    C_TaskStatus_Stopping = 6,
    C_TaskStatus_Stopped = 7,
    C_TaskStatus_Success = 8,
    C_TaskStatus_Error = 9,
    C_TaskStatus_NotExecuted = 10,
} C_TaskStatus;

typedef struct C_Logger C_Logger;

C_Logger* rust_create_logger();
void rust_free_logger(C_Logger*);
void rust_logger_log(C_Logger*, const char*);
void rust_logger_set_status(C_Logger*, C_TaskStatus);
C_TaskStatus rust_logger_get_status(C_Logger*);
*/
import "C"
import "unsafe"

type TaskStatus int

const (
    TaskWaitingStatus TaskStatus = TaskStatus(C.C_TaskStatus_Waiting)
    TaskStartingStatus TaskStatus = TaskStatus(C.C_TaskStatus_Starting)
    TaskWaitingConfirmation TaskStatus = TaskStatus(C.C_TaskStatus_WaitingConfirmation)
    TaskConfirmed TaskStatus = TaskStatus(C.C_TaskStatus_Confirmed)
    TaskRejected TaskStatus = TaskStatus(C.C_TaskStatus_Rejected)
    TaskRunningStatus TaskStatus = TaskStatus(C.C_TaskStatus_Running)
    TaskStoppingStatus TaskStatus = TaskStatus(C.C_TaskStatus_Stopping)
    TaskStoppedStatus TaskStatus = TaskStatus(C.C_TaskStatus_Stopped)
    TaskSuccessStatus TaskStatus = TaskStatus(C.C_TaskStatus_Success)
    TaskFailStatus TaskStatus = TaskStatus(C.C_TaskStatus_Error)
    TaskNotExecutedStatus TaskStatus = TaskStatus(C.C_TaskStatus_NotExecuted)
)

type Logger struct {
    ptr *C.C_Logger
}

func NewLogger() *Logger {
    return &Logger{ptr: C.rust_create_logger()}
}

func (l *Logger) Log(msg string) {
    cMsg := C.CString(msg)
    defer C.free(unsafe.Pointer(cMsg))
    C.rust_logger_log(l.ptr, cMsg)
}

func (l *Logger) SetStatus(status TaskStatus) {
    C.rust_logger_set_status(l.ptr, C.C_TaskStatus(status))
}

func (l *Logger) GetStatus() TaskStatus {
    return TaskStatus(C.rust_logger_get_status(l.ptr))
}

func (l *Logger) Close() {
    if l.ptr != nil {
        C.rust_free_logger(l.ptr)
        l.ptr = nil
    }
}
```

**ВАРИАНТ B: Полное удаление (рекомендуется для чистого Rust)**

Если вы полностью переходите на Rust, просто удалите модуль и используйте Rust бинарник.

### Шаг 1.3: Удаление модуля

```bash
# Удалить pkg/task_logger
rm -rf pkg/task_logger

# Проверить компиляцию (если остался Go код)
go build ./...

# Если есть ошибки, обновите импорты на Rust FFI
```

---

## 🗑 Этап 2: Удаление pkg/ssh

### Шаг 2.1: Анализ зависимостей

```bash
# Найти все файлы, использующие pkg/ssh
grep -r "\"github.com/semaphoreui/semaphore/pkg/ssh\"" --include="*.go" . | cut -d: -f1 | sort -u
```

**Ожидаемый результат** (7 файлов):
```
./db_lib/AccessKeyInstaller.go
./db_lib/CmdGitClient.go
./services/schedules/SchedulePool_test.go
./services/server/access_key_installation_svc.go
./services/tasks/LocalJob.go
./services/tasks/TaskRunner_test.go
./cli/cmd/runner.go
```

### Шаг 2.2: Подготовка замены

**ВАРИАНТ A: Использование Rust через FFI**

```go
// pkg/rustlib/ssh_agent.go
package rustlib

/*
#cgo LDFLAGS: -L${SRCDIR}/lib -lsemaphore_ffi
#include <stdlib.h>
#include <stdint.h>
#include <stdbool.h>

typedef enum {
    C_AccessKeyRole_Git = 0,
    C_AccessKeyRole_AnsiblePasswordVault = 1,
    C_AccessKeyRole_AnsibleBecomeUser = 2,
    C_AccessKeyRole_AnsibleUser = 3,
} C_AccessKeyRole;

typedef enum {
    C_AccessKeyType_Ssh = 0,
    C_AccessKeyType_LoginPassword = 1,
    C_AccessKeyType_None = 2,
} C_AccessKeyType;

typedef struct {
    int64_t id;
    C_AccessKeyType key_type;
    const char* private_key;
    const char* passphrase;
    const char* login;
    const char* password;
    int64_t project_id;
} C_AccessKey;

typedef struct {
    bool has_ssh_agent;
    const char* login;
    const char* password;
    const char* error;
} C_AccessKeyInstallation;

C_AccessInstallation rust_install_access_key(
    const C_AccessKey*,
    C_AccessKeyRole,
    void* logger
);
void rust_free_access_key_installation(C_AccessKeyInstallation*);
*/
import "C"
import "unsafe"

type AccessKeyRole int

const (
    AccessKeyRoleGit AccessKeyRole = AccessKeyRole(C.C_AccessKeyRole_Git)
    AccessKeyRoleAnsiblePasswordVault AccessKeyRole = AccessKeyRole(C.C_AccessKeyRole_AnsiblePasswordVault)
    AccessKeyRoleAnsibleBecomeUser AccessKeyRole = AccessKeyRole(C.C_AccessKeyRole_AnsibleBecomeUser)
    AccessKeyRoleAnsibleUser AccessKeyRole = AccessKeyRole(C.C_AccessKeyRole_AnsibleUser)
)

type AccessKeyType int

const (
    AccessKeySSH AccessKeyType = AccessKeyType(C.C_AccessKeyType_Ssh)
    AccessKeyLoginPassword AccessKeyType = AccessKeyType(C.C_AccessKeyType_LoginPassword)
    AccessKeyNone AccessKeyType = AccessKeyType(C.C_AccessKeyType_None)
)

type AccessKey struct {
    ID         int64
    Type       AccessKeyType
    PrivateKey string
    Passphrase string
    Login      string
    Password   string
    ProjectID  *int64
}

type AccessKeyInstallation struct {
    SSHAgent *SSHAgent
    Login    string
    Password string
}

type SSHAgent struct {
    SocketFile string
}

type KeyInstaller struct{}

func (KeyInstaller) Install(key AccessKey, usage AccessKeyRole, logger *Logger) (AccessKeyInstallation, error) {
    cKey := C.C_AccessKey{
        id:         C.int64_t(key.ID),
        key_type:   C.C_AccessKeyType(key.Type),
        private_key: C.CString(key.PrivateKey),
        passphrase: C.CString(key.Passphrase),
        login:      C.CString(key.Login),
        password:   C.CString(key.Password),
        project_id: C.int64_t(0),
    }
    
    if key.ProjectID != nil {
        cKey.project_id = C.int64_t(*key.ProjectID)
    }
    
    defer C.free(unsafe.Pointer(cKey.private_key))
    defer C.free(unsafe.Pointer(cKey.passphrase))
    defer C.free(unsafe.Pointer(cKey.login))
    defer C.free(unsafe.Pointer(cKey.password))
    
    var loggerPtr unsafe.Pointer
    if logger != nil {
        loggerPtr = unsafe.Pointer(logger.ptr)
    }
    
    cResult := C.rust_install_access_key(&cKey, C.C_AccessKeyRole(usage), loggerPtr)
    defer C.rust_free_access_key_installation(&cResult)
    
    if cResult.error != nil {
        errMsg := C.GoString(cResult.error)
        return AccessKeyInstallation{}, fmt.Errorf("%s", errMsg)
    }
    
    installation := AccessKeyInstallation{
        Login:    C.GoString(cResult.login),
        Password: C.GoString(cResult.password),
    }
    
    if cResult.has_ssh_agent {
        installation.SSHAgent = &SSHAgent{}
    }
    
    return installation, nil
}
```

### Шаг 2.3: Удаление модуля

```bash
# Удалить pkg/ssh
rm -rf pkg/ssh

# Проверить компиляцию
go build ./...
```

---

## ✅ Этап 3: Финальная проверка

### 3.1: Проверка Rust

```bash
cd rust

# Сборка
cargo build --release

# Тесты
cargo test

# Должно быть: 125 тестов прошли
```

### 3.2: Проверка Go (если остался)

```bash
# Тесты
go test ./...

# Сборка
go build -o semaphore ./cli
```

### 3.3: Проверка бинарника

```bash
# Rust бинарник
./rust/target/release/semaphore version

# Ожидаемый вывод:
# semaphore version 0.1.0
```

---

## 🔧 Альтернатива: Полная замена Go на Rust

Если вы хотите полностью отказаться от Go:

### Шаг 1: Обновите скрипты запуска

**Было** (Go):
```bash
./semaphore server --config config.json
```

**Стало** (Rust):
```bash
./rust/target/release/semaphore server --config config.json
```

### Шаг 2: Обновите Dockerfile

```dockerfile
# Было (Go)
FROM golang:1.21 AS builder
COPY . /src
RUN cd /src && go build -o semaphore ./cli

# Стало (Rust)
FROM rust:1.75 AS builder
COPY rust /src
RUN cd /src && cargo build --release

FROM debian:bookworm-slim
COPY --from=builder /src/target/release/semaphore /usr/local/bin/
```

### Шаг 3: Обновите docker-compose.yml

```yaml
# Было
services:
  server:
    build:
      context: .
      dockerfile: Dockerfile
    
# Стало
services:
  server:
    build:
      context: ./rust
      dockerfile: Dockerfile
```

---

## 📊 Ожидаемые результаты

### После удаления Go модулей:

| Метрика | До | После |
|---------|----|----|
| **Языки** | Go + Rust | Только Rust |
| **Компиляторы** | Go + Rust | Только Rust |
| **Размер** | ~50 MB (Go) | ~5-10 MB (Rust) |
| **Память** | ~50-100 MB | ~10-30 MB |
| **Запуск** | ~1-2 сек | ~0.1-0.5 сек |

---

## ⚠️ Возможные проблемы и решения

### Проблема 1: Ошибки компиляции Go

**Ошибка**:
```
package github.com/semaphoreui/semaphore/pkg/task_logger: no required module provides package
```

**Решение**:
- Либо используйте Rust FFI (см. выше)
- Либо полностью удалите Go код и используйте только Rust

### Проблема 2: Rust не находит зависимости

**Ошибка**:
```
error: package `sqlx v0.8.0` cannot be built because it requires rustc 1.75
```

**Решение**:
```bash
# Обновите Rust
rustup update stable

# Проверьте версию
rustc --version  # Должно быть 1.75+
```

### Проблема 3: FFI библиотеки не найдены

**Ошибка**:
```
error while loading shared libraries: libsemaphore_ffi.so: cannot open shared object file
```

**Решение**:
```bash
# Скопируйте библиотеку в системную директорию
sudo cp rust/target/release/libsemaphore_ffi.so /usr/lib/

# Обновите кэш библиотек
sudo ldconfig
```

---

## 📞 Поддержка

Если возникли проблемы:

1. **Проверьте документацию**:
   - `rust/README.md`
   - `rust/MIGRATION_REPORT.md`
   - `rust/MIGRATION.md`

2. **Запустите тесты**:
   ```bash
   cd rust && cargo test -- --nocapture
   ```

3. **Проверьте логи**:
   ```bash
   RUST_LOG=debug ./target/release/semaphore server
   ```

4. **Откройте issue**:
   - https://github.com/alexandervashurin/semaphore/issues

---

## ✅ Чеклист успешного удаления

- [ ] Сделана резервная копия Go модулей
- [ ] Rust код компилируется (`cargo build --release`)
- [ ] Все 125 тестов проходят
- [ ] FFI библиотеки собраны (если используются)
- [ ] Go код обновлён для использования FFI (если нужно)
- [ ] `pkg/task_logger` удалён
- [ ] `pkg/ssh` удалён
- [ ] Go код компилируется (если остался)
- [ ] Бинарник работает (`semaphore version`)
- [ ] Документация обновлена

---

**Ответственный**: Alexander Vashurin  
**Дата**: 2026-02-26  
**Статус**: ✅ ГОТОВО К ВЫПОЛНЕНИЮ
