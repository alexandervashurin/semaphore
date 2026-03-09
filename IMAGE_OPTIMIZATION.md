# 🐳 Оптимизация размера Docker образов

> **Цель:** Уменьшение размера образов с ~450 MB до ~30-80 MB без потери функциональности

---

## 📊 Сравнение образов

| Образ | Размер | База | Время запуска | Совместимость |
|-------|--------|------|---------------|---------------|
| **Standard** | ~450 MB | Debian bookworm | ~3 сек | ⭐⭐⭐⭐⭐ |
| **Slim** | ~180 MB | Debian bookworm-slim | ~2 сек | ⭐⭐⭐⭐⭐ |
| **Alpine** | ~60 MB | Alpine 3.19 | ~1 сек | ⭐⭐⭐⭐ |
| **Distroless** | ~35 MB | Distroless cc | ~1 сек | ⭐⭐⭐ |

---

## 🚀 Быстрый старт

### Slim образ (рекомендуется)

```bash
# Сборка
docker build -f deployment/Dockerfile.slim -t semaphore:slim .

# Запуск
docker run -d \
  --name semaphore \
  -p 80:3000 \
  -v semaphore_data:/app/data \
  semaphore:slim
```

### Alpine образ (минимальный размер)

```bash
# Сборка
docker build -f deployment/Dockerfile.alpine -t semaphore:alpine .

# Запуск
docker run -d \
  --name semaphore \
  -p 80:3000 \
  -v semaphore_data:/app/data \
  semaphore:alpine
```

### Distroless образ (максимальная безопасность)

```bash
# Сборка
docker build -f deployment/Dockerfile.distroless -t semaphore:distroless .

# Запуск
docker run -d \
  --name semaphore \
  -p 80:3000 \
  -v semaphore_data:/app/data \
  semaphore:distroless
```

---

## 🔧 Автоматическая сборка всех образов

```bash
# Сборка всех вариантов
./scripts/build-optimized-images.sh latest

# Сборка с push в registry
./scripts/build-optimized-images.sh v0.1.0 ghcr.io/alexandervashurin
```

---

## 📦 Техники оптимизации

### 1. Multi-stage сборка

```dockerfile
FROM rust:1.88-slim AS builder
# ... сборка ...

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/semaphore /usr/local/bin/
```

**Эффект:** Удаление всех зависимостей сборки (~300 MB)

### 2. Статическая линковка (musl)

```dockerfile
FROM rust:1.88-alpine AS musl-builder
RUN apk add musl-dev musl-tools
ENV RUSTFLAGS="-C target-feature=+crt-static"
RUN cargo build --release --target x86_64-unknown-linux-musl
```

**Эффект:** Удаление зависимостей runtime (~100 MB)

### 3. Strip бинарника

```dockerfile
RUN strip --strip-all target/release/semaphore
```

**Эффект:** Удаление отладочной информации (~50 MB)

### 4. Оптимизация Cargo профиля

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = "z"  # Оптимизация размера
panic = "abort"   # Меньше бинарник
```

**Эффект:** Уменьшение бинарника на 30-50%

### 5. Минимальные runtime зависимости

```dockerfile
# Вместо полного набора
RUN apt-get install -y ca-certificates libssl3

# Только необходимое
RUN apk add --no-cache ca-certificates libgcc libstdc++
```

**Эффект:** Удаление лишних пакетов (~100 MB)

### 6. Distroless base

```dockerfile
FROM gcr.io/distroless/cc-debian12
```

**Эффект:** Нет shell, нет package manager, только приложение (~20 MB)

---

## 📈 Профили сборки

### Release (стандартный)

```bash
cargo build --release
```

- LTO: true
- Codegen units: 1
- Strip: true
- Opt-level: z

### Release-small (минимальный размер)

```bash
cargo build --profile release-small
```

- Наследует release
- Дополнительная оптимизация размера

### Release-fast (максимальная производительность)

```bash
cargo build --profile release-fast
```

- LTO: fat
- Opt-level: 3
- Для production с высокими требованиями

---

## 🔍 Анализ размера

### Docker layers

```bash
# Просмотр слоёв
docker history semaphore:slim

# Детальный анализ
docker inspect semaphore:slim
```

### Image size

```bash
# Размер образа
docker images semaphore

# Сравнение
docker images | grep semaphore
```

### Bina
