# 🔧 Решение проблем с CRUD

## ❗ Проблема: Не работает редактирование проектов

### Симптомы:
- Нажимаете ✏️ на проекте, но ничего не происходит
- Или видите сообщение "Функция редактирования в разработке"

### Решение:

**1. Очистите кэш браузера**

Браузер кэширует старую версию JavaScript файла.

**Chrome/Edge:**
- Нажмите `Ctrl+Shift+Delete` (Windows/Linux) или `Cmd+Shift+Delete` (Mac)
- Выберите "Кэшированные изображения и файлы"
- Нажмите "Удалить"

**Firefox:**
- Нажмите `Ctrl+Shift+Delete` (Windows/Linux) или `Cmd+Shift+Delete` (Mac)
- Выберите "Кэш"
- Нажмите "OK"

**Или жёсткая перезагрузка:**
- Нажмите `Ctrl+F5` (Windows/Linux) или `Cmd+Shift+R` (Mac)

**2. Проверьте версию JS файла**

Откройте консоль разработчика (F12) и выполните:

```javascript
console.log('Edit function:', typeof editProject);
```

Должно вывести: `function`

Если выводит: `undefined` - страница использует старую версию.

**3. Проверьте консоль на ошибки**

Откройте консоль (F12) и посмотрите на ошибки при нажатии ✏️.

---

## ❗ Проблема: БД не заполнена тестовыми данными

### Симптомы:
- Пустой список проектов
- Нет шаблонов, инвентарей и т.д.

### Решение:

**1. Проверьте наполнение БД**

```bash
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT COUNT(*) FROM project;"
```

Должно вывести: `4`

**2. Если БД пуста, пересоздайте контейнеры**

```bash
./start.sh --clean
```

Или вручную:

```bash
docker-compose down -v
docker-compose up -d
```

**3. Проверьте логи БД**

```bash
docker logs semaphore-db | grep -i "demo"
```

Должны быть сообщения об инициализации демо-данных.

---

## ❗ Проблема: Ошибка при сохранении проекта

### Симптомы:
- Заполняете форму редактирования
- Нажимаете "Сохранить"
- Видите ошибку

### Решение:

**1. Проверьте токен авторизации**

Откройте консоль (F12) и выполните:

```javascript
console.log('Token:', TOKEN);
```

Если `undefined` - вы не авторизованы. Войдите заново.

**2. Проверьте API**

```bash
curl http://localhost:80/api/projects \
  -H "Authorization: Bearer YOUR_TOKEN"
```

**3. Проверьте backend**

```bash
curl http://localhost:3000/api/health
```

Должно вывести: `OK`

---

## 📊 Проверка наполнения БД

Выполните команды для проверки:

```bash
# Проекты
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, name FROM project;"

# Шаблоны
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, name, playbook FROM template LIMIT 5;"

# Инвентари
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, name FROM inventory;"

# Репозитории
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, name FROM repository;"

# Окружения
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, name FROM environment;"

# Ключи
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, name, type FROM access_key;"

# Пользователи
docker exec semaphore-db psql -U semaphore -d semaphore -c "SELECT id, username, name FROM \"user\";"
```

**Ожидаемые результаты:**
- Проекты: 4
- Шаблоны: 12
- Инвентари: 5
- Репозитории: 5
- Окружения: 5
- Ключи: 5
- Пользователи: 4

---

## ✅ Всё работает, но...

### Если редактирование работает через API, но не через UI:

1. **Очистите кэш браузера** (см. выше)
2. **Проверьте консоль на ошибки** (F12 → Console)
3. **Перезагрузите страницу** с Ctrl+F5
4. **Выйдите и войдите заново**

### Если видите старое сообщение "в разработке":

Это значит браузер использует кэшированную версию JS. Очистите кэш!

---

## 📝 Тестовые данные

**Пользователи:**
- admin / demo123
- john.doe / demo123
- jane.smith / demo123
- devops / demo123

**Проекты:**
1. Demo Infrastructure
2. Web Application Deployment
3. Database Management
4. Security & Compliance

Все проекты имеют заполненные поля:
- ✅ name
- ✅ alert
- ✅ max_parallel_tasks
- ✅ type
- ✅ alert_chat (null)
- ✅ default_secret_storage_id (null)

---

## 🆘 Если ничего не помогло

1. Пересоздайте контейнеры:
   ```bash
   ./cleanup.sh --all
   ./start.sh
   ```

2. Проверьте логи:
   ```bash
   docker logs semaphore-db --tail 50
   docker logs semaphore-frontend --tail 50
   ```

3. Проверьте backend:
   ```bash
   ps aux | grep semaphore
   curl http://localhost:3000/api/health
   ```

4. Откройте issue на GitHub с логами.
