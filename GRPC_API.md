# gRPC API в Semaphore UI

> **Внутренний gRPC API для межсервисного взаимодействия**

## 📖 Оглавление

- [Обзор](#обзор)
- [Архитектура](#архитектура)
- .proto файлы](#proto-файлы)
- [Сервисы](#сервисы)
- [Конфигурация](#конфигурация)
- [Использование](#использование)

---

## 📋 Обзор

gRPC API в Semaphore UI предоставляет высокопроизводительный интерфейс для:

- **Внутреннего взаимодействия** между компонентами системы
- **Распределённого выполнения задач** через раннеры
- **Кэширования и инвалидации** данных
- **Мониторинга и телеметрии**

**Преимущества:**

| Преимущество | Описание |
|--------------|----------|
| **Производительность** | HTTP/2 + Protobuf = низкая задержка |
| **Типобезопасность** | Строгая типизация через .proto |
| **Кроссплатформенность** | Клиенты на любом языке с gRPC поддержкой |
| **Streaming** | Поддержка bidirectional streaming |

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Semaphore UI                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │              gRPC Server                          │  │
│  │  Port: 50051                                    │  │
│  │  ┌─────────────────────────────────────────────┐│  │
│  │  │  TaskService                                ││  │
│  │  │  • RunTask                                  ││  │
│  │  │  • StopTask                                 ││  │
│  │  │  • GetTaskStatus                            ││  │
│  │  │  • StreamTaskStatus                         ││  │
│  │  └─────────────────────────────────────────────┘│  │
│  │  ┌─────────────────────────────────────────────┐│  │
│  │  │  ProjectService                             ││  │
│  │  │  • GetProject                               ││  │
│  │  │  • ListProjects                             ││  │
│  │  │  • InvalidateProjectCache                   ││  │
│  │  │  • GetProjectStatistics                     ││  │
│  │  └─────────────────────────────────────────────┘│  │
│  │  ┌─────────────────────────────────────────────┐│  │
│  │  │  RunnerService                              ││  │
│  │  │  • RegisterRunner                           ││  │
│  │  │  • SendHeartbeat                            ││  │
│  │  │  • GetPendingTasks                          ││  │
│  │  │  • ReportTaskResult                         ││  │
│  │  └─────────────────────────────────────────────┘│  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

---

## 📝 .proto файлы

### Базовые типы

```protobuf
// Статус задачи
enum TaskStatus {
    TASK_STATUS_UNSPECIFIED = 0;
    TASK_STATUS_PENDING = 1;
    TASK_STATUS_RUNNING = 2;
    TASK_STATUS_SUCCESS = 3;
    TASK_STATUS_FAILED = 4;
    TASK_STATUS_STOPPED = 5;
}

// Информация о задаче
message TaskInfo {
    int64 id = 1;
    int64 template_id = 2;
    int64 project_id = 3;
    TaskStatus status = 4;
    string output = 5;
    int32 exit_code = 6;
    optional string commit_hash = 7;
    int64 created_at = 8;
    optional int64 started_at = 9;
    optional int64 ended_at = 10;
}
```

### TaskService

```protobuf
service TaskService {
    // Запустить задачу
    rpc RunTask(RunTaskRequest) returns (RunTaskResponse);
    
    // Остановить задачу
    rpc StopTask(StopTaskRequest) returns (StopTaskResponse);
    
    // Получить статус задачи
    rpc GetTaskStatus(GetTaskStatusRequest) returns (TaskInfo);
    
    // Получить список задач проекта
    rpc ListProjectTasks(ListProjectTasksRequest) returns (ListProjectTasksResponse);
    
    // Stream статуса выполнения задачи
    rpc StreamTaskStatus(StreamTaskStatusRequest) returns (stream TaskStatusUpdate);
}
```

### ProjectService

```protobuf
service ProjectService {
    // Получить проект
    rpc GetProject(GetProjectRequest) returns (ProjectInfo);
    
    // Получить список проектов
    rpc ListProjects(ListProjectsRequest) returns (ListProjectsResponse);
    
    // Обновить кэш проекта
    rpc InvalidateProjectCache(InvalidateProjectCacheRequest) returns (Empty);
    
    // Получить статистику проекта
    rpc GetProjectStatistics(GetProjectStatisticsRequest) returns (ProjectStatistics);
}
```

### RunnerService

```protobuf
service RunnerService {
    // Зарегистрировать раннер
    rpc RegisterRunner(RegisterRunnerRequest) returns (RegisterRunnerResponse);
    
    // Отправить heartbeat
    rpc SendHeartbeat(SendHeartbeatRequest) returns (Empty);
    
    // Получить задачи для выполнения
    rpc GetPendingTasks(GetPendingTasksRequest) returns (GetPendingTasksResponse);
    
    // Отчёт о выполнении задачи
    rpc ReportTaskResult(ReportTaskResultRequest) returns (Empty);
}
```

---

## 🔌 Сервисы

### TaskService

**Назначение:** Управление задачами Semaphore

| Метод | Вход | Выход | Описание |
|-------|------|-------|----------|
| `RunTask` | `RunTaskRequest` | `RunTaskResponse` | Запуск задачи |
| `StopTask` | `StopTaskRequest` | `StopTaskResponse` | Остановка задачи |
| `GetTaskStatus` | `GetTaskStatusRequest` | `TaskInfo` | Статус задачи |
| `ListProjectTasks` | `ListProjectTasksRequest` | `ListProjectTasksResponse` | Список задач |
| `StreamTaskStatus` | `StreamTaskStatusRequest` | `stream TaskStatusUpdate` | Stream статуса |

### ProjectService

**Назначение:** Управление проектами

| Метод | Вход | Выход | Описание |
|-------|------|-------|----------|
| `GetProject` | `GetProjectRequest` | `ProjectInfo` | Информация о проекте |
| `ListProjects` | `ListProjectsRequest` | `ListProjectsResponse` | Список проектов |
| `InvalidateProjectCache` | `InvalidateProjectCacheRequest` | `Empty` | Инвалидация кэша |
| `GetProjectStatistics` | `GetProjectStatisticsRequest` | `ProjectStatistics` | Статистика |

### RunnerService

**Назначение:** Управление раннерами

| Метод | Вход | Выход | Описание |
|-------|------|-------|----------|
| `RegisterRunner` | `RegisterRunnerRequest` | `RegisterRunnerResponse` | Регистрация |
| `SendHeartbeat` | `SendHeartbeatRequest` | `Empty` | Heartbeat |
| `GetPendingTasks` | `GetPendingTasksRequest` | `GetPendingTasksResponse` | Pending задачи |
| `ReportTaskResult` | `ReportTaskResultRequest` | `Empty` | Результат задачи |

---

## ⚙️ Конфигурация

### Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SEMAPHORE_GRPC_ENABLED` | Включить gRPC сервер | `false` |
| `SEMAPHORE_GRPC_ADDRESS` | Адрес gRPC сервера | `0.0.0.0:50051` |
| `SEMAPHORE_GRPC_MAX_MESSAGE_SIZE` | Макс. размер сообщения | `4194304` (4MB) |

### Конфигурационный файл

```json
{
  "grpc": {
    "enabled": true,
    "address": "0.0.0.0:50051",
    "max_message_size": 4194304,
    "enable_reflection": true
  }
}
```

---

## 💡 Использование

### Запуск gRPC сервера

```rust
use semaphore::grpc::{GrpcServer, GrpcServerConfig};

#[tokio::main]
async fn main() {
    let config = GrpcServerConfig {
        address: "0.0.0.0:50051".parse().unwrap(),
        enable_reflection: true,
        max_message_size: 4 * 1024 * 1024,
    };
    
    let server = GrpcServer::new(config);
    server.serve().await.unwrap();
}
```

### gRPC клиент (Python пример)

```python
import grpc
import semaphore_pb2
import semaphore_pb2_grpc

def run_task():
    channel = grpc.insecure_channel('localhost:50051')
    stub = semaphore_pb2_grpc.TaskServiceStub(channel)
    
    request = semaphore_pb2.RunTaskRequest(
        template_id=1,
        project_id=1
    )
    
    response = stub.RunTask(request)
    print(f"Task ID: {response.task_id}")
    print(f"Status: {response.status}")
```

### gRPC клиент (Go пример)

```go
package main

import (
    "context"
    "google.golang.org/grpc"
    pb "semaphore/grpc"
)

func main() {
    conn, _ := grpc.Dial("localhost:50051", grpc.WithInsecure())
    client := pb.NewTaskServiceClient(conn)
    
    resp, _ := client.RunTask(context.Background(), &pb.RunTaskRequest{
        TemplateId: 1,
        ProjectId:  1,
    })
    
    println("Task ID:", resp.TaskId)
}
```

---

## 🔧 Компиляция .proto файлов

### Требования

- `protoc` - Protocol Buffers compiler
- `tonic-build` - Rust gRPC code generator

### Установка protoc

```bash
# Ubuntu/Debian
apt-get install protobuf-compiler

# macOS
brew install protobuf

# Windows
choco install protobuf
```

### Генерация кода

```bash
cd rust
cargo build
# tonic-build автоматически сгенерирует код из .proto файлов
```

### Структура сгенерированного кода

```
rust/src/grpc/
├── semaphore.rs          # Сгенерированные типы и сервисы
├── semaphore_descriptor.bin # File descriptor set для reflection
├── server.rs             # Реализация сервера
└── services.rs           # Реализация сервисов
```

---

## 📊 Метрики

### Prometheus метрики

```
# gRPC метрики
semaphore_grpc_requests_total{service, method, status}
semaphore_grpc_request_duration_seconds{service, method}
semaphore_grpc_active_streams{service}
```

---

## 🔗 Ссылки

- [gRPC Documentation](https://grpc.io/docs/)
- [Tonic Documentation](https://docs.rs/tonic)
- [Protocol Buffers](https://developers.google.com/protocol-buffers)
- [proto/semaphore.proto](rust/proto/semaphore.proto)

---

*Последнее обновление: 10 марта 2026 г.*
