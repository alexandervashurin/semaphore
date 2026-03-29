# 🎛️ Голландский Штурвал — План реализации Kubernetes UI

> **Долгосрочная цель:** максимально полное покрытие сценариев Kubernetes в Web UI (ориентир — эквивалент `kubectl` + хороший UX).  
> **MVP и промежуточные релизы** — явный scope по фазам ниже; «100%» не блокирует первый релиз модуля.  
> **Backend:** Rust (Axum + REST `/api/kubernetes/...`; kube-rs) | **Frontend:** Vanilla JS

---

## 📋 Оглавление

1. [Обзор проекта](#обзор-проекта)
2. [Архитектура](#архитектура)
3. [Текущее состояние репозитория](#текущее-состояние-репозитория)
4. [Эволюция клиента (kubectl → kube-rs)](#эволюция-клиента-kubectl--kube-rs)
5. [Безопасность и мульти-тенантность](#безопасность-и-мульти-тенантность)
6. [Референсные продукты и лучшие практики Web UI](#ref-web-ui-best-practices)
7. [Фазы реализации](#фазы-реализации)
8. [Детальный план по ресурсам](#детальный-план-по-ресурсам)
9. [Дополнительные ресурсы (outline)](#дополнительные-ресурсы-outline)
10. [API Reference](#api-reference)
11. [Frontend компоненты](#frontend-компоненты)
12. [Интеграции](#интеграции)
13. [Тестирование](#тестирование)
14. [KPI и нефункциональные требования](#kpi-и-нефункциональные-требования)
15. [Документация](#документация)
16. [Changelog](#changelog)
17. [Референсы](#референсы)
18. [Следующие шаги](#следующие-шаги)

---

## 🎯 Обзор проекта

### Цель
Создать **полнофункциональный Web UI для Kubernetes**, который покрывает 100% возможностей `kubectl` и лучших практик из:
- **Headlamp** — официальный Kubernetes SIG UI
- **Lens Desktop** — IDE для Kubernetes
- **Octant** — визуализация зависимостей
- **K9s** — terminal UX паттерны
- **Rancher** — enterprise управление

Детальный разбор референсов, индустриальных практик (dry-run, SSA, RBAC-UX) и **backlog пробелов** — в разделе [Референсные продукты и лучшие практики Web UI](#ref-web-ui-best-practices).

### Название
**"Голландский штурвал"** (Dutch Helm) — символизирует полный контроль над кластером, как капитан корабля управляет судном с помощью штурвала.

### Ключевые принципы
1. **Без обязательности CLI для пользователя** — типовые операции из UI; на стороне сервера допустимы `kubectl`/Helm CLI там, где это временно или осознанно ([эволюция клиента](#эволюция-клиента-kubectl--kube-rs)).
2. **Real-time** — WebSocket для live обновлений
3. **Visual First** — графы, топологии, диаграммы
4. **Safe by Default** — подтверждение деструктивных операций
5. **Multi-cluster** — управление несколькими кластерами (модель контекстов закладывается с **фазы 1**, полный UI — позже)

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────────────┐
│                     Frontend (Vanilla JS)                        │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐          │
│  │ Cluster  │ │Workloads │ │  Config  │ │  Admin   │          │
│  │Overview  │ │  Manager │ │  & Net   │ │  Panel   │          │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘          │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              WebSocket (Real-time Events)                 │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ HTTP/WS API
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Backend (Rust + Axum)                         │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               API Handlers (REST; GraphQL — опционально)   │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │               Kubernetes Controller Layer                 │  │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐            │  │
│  │  │ Inform │ │ Watch  │ │ Cache  │ │ Queue  │            │  │
│  │  │ er     │ │ Stream │ │ Store  │ │        │            │  │
│  │  └────────┘ └────────┘ └────────┘ └────────┘            │  │
│  └──────────────────────────────────────────────────────────┘  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │              kube-rs Client Layer                         │  │
│  │  • k8s-openapi (v1_30)                                    │  │
│  │  • kube (0.98) + kube-runtime                             │  │
│  │  • kubelet (опционально)                                  │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                              │
                              │ Kubernetes API
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Kubernetes Cluster                            │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
│  │  Pods   │ │ Deploy  │ │  Svc    │ │  Config │              │
│  │         │ │ ments   │ │         │ │  Maps   │              │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘              │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐              │
│  │ Secrets │ │  Jobs   │ │ Ingress │ │  RBAC   │              │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘              │
└─────────────────────────────────────────────────────────────────┘
```

**Примечание.** Слои «Kubernetes Controller / Informer» и «kube-rs Client» в схеме выше описывают **целевое состояние** полноценного UI. См. [текущее состояние репозитория](#текущее-состояние-репозитория) ниже.

**GraphQL:** в репозитории уже есть модуль [rust/src/api/graphql/](rust/src/api/graphql/) — для K8s UI **основной контракт — REST** (`/api/kubernetes/...`). GraphQL подключать только при явной потребности (агрегации, меньше round-trip к браузеру), иначе дублирование контракта.

**Версии клиента:** при смене целевой версии Kubernetes обновлять **согласованно** `k8s-openapi` (feature на API, e.g. `v1_30`) и `kube`; фиксировать целевую минорную версию кластера в CI (kind) и в release notes.

---

## 📌 Текущее состояние репозитория

Полноценный REST-префикс `/api/kubernetes/...` и отдельный K8s dashboard в этом документе — **планируемая подсистема** поверх существующего продукта Velum ([MASTER_PLAN.md](MASTER_PLAN.md)). Сейчас в коде уже есть инфраструктурная интеграция с Kubernetes **без** описанного ниже HTTP API.

| Компонент | Путь в репозитории | Назначение | Статус |
|-----------|---------------------|------------|--------|
| Обёртка `kubectl` | [rust/src/kubernetes/client.rs](rust/src/kubernetes/client.rs) | Вызовы `kubectl` (apply, get и т.д.) | Реализовано |
| Конфиг / Job runner | [rust/src/kubernetes/config.rs](rust/src/kubernetes/config.rs), [job.rs](rust/src/kubernetes/job.rs) | Запуск задач как K8s Job; типы Pod/Job через `k8s_openapi` | Реализовано |
| Helm | [rust/src/kubernetes/helm.rs](rust/src/kubernetes/helm.rs) | Операции Helm CLI | Реализовано |
| Зависимости | [rust/Cargo.toml](rust/Cargo.toml): `kube`, `kube-runtime`, `k8s-openapi` | Готовность к API-клиенту | `kube` / `kube-runtime` **пока не использованы** в коде (`use kube::` отсутствует) |
| HTTP handlers | `rust/src/api/handlers/kubernetes/*.rs` | Публичный K8s UI API | **Не заведён** — маршруты из раздела «Детальный план» предстоит реализовать |

**Вывод:** full K8s UI — крупная новая подсистема; переиспользуются существующие **JWT/auth**, стиль **Material / [web/public/](web/public/)**. Отдельный GraphQL-слой для K8s — опционально (см. [архитектуру](#архитектура)).

---

## 🔄 Эволюция клиента (kubectl → kube-rs)

Сохраняем рабочий `kubectl`/`Helm` там, где это оправдано; для UI и watch — поэтапно вводим **`kube::Client`**.

```mermaid
flowchart LR
  subgraph today [Сейчас]
    Kubectl[kubectl subprocess]
    TaskJob[Job runner Helm]
  end
  subgraph phaseA [Фаза A]
    KubeClient[kube Client Api]
    ListWatch[list watch базовых ресурсов]
  end
  subgraph phaseB [Фаза B]
    Informer[informer watcher]
    Ws[WebSocket Axum]
  end
  today --> phaseA --> phaseB
```

- **Фаза A:** общий сервис (например `KubernetesClusterService`) на `kube::Client` — kubeconfig и in-cluster; list/get для Namespaces, Pods, Deployments и т.д. Параллельно не ломать текущий [KubernetesClient](rust/src/kubernetes/client.rs) для сценариев `kubectl`/`helm`.
- **Фаза B:** для real-time — `Watch` / informer + каналы в WebSocket (события, статусы подов).
- **Exec / port-forward / attach:** отдельные риски (протокол, безопасность, таймауты); предпочтительна **прокси-схема через backend** с явной авторизацией и аудитом. См. [Безопасность](#безопасность-и-мульти-тенантность).

---

## 🔐 Безопасность и мульти-тенантность

- **Credentials:** не хранить kubeconfig в БД в открытом виде. Варианты: шифрованный blob, внешний secret store, для in-cluster — OIDC/IRSA (или аналог) вместо длительных статических токенов там, где это возможно.
- **Модель доступа:** пользователь Velum ≠ ServiceAccount кластера. Нужна матрица «роль/проект в Velum → разрешённые verbs / resources / namespaces (или кластеры)» и проверка на каждом маршруте `/api/kubernetes/...`.
- **Аудит:** фиксировать деструктивные действия (Delete, Rollback, uninstall Helm и т.д.) с привязкой к пользователю Velum; по возможности использовать существующий audit log приложения.
- **Multi-cluster (архитектура с первой фазы):** модель «подключение кластера» (id, display name, способ аутентификации), изоляция контекста в запросах, без утечки kubeconfig между сессиями — даже если UI переключателя кластеров появится в фазе 10.
- **Exec / port-forward / attach:** жёсткие **таймауты** сессии и **лимиты** на число параллельных сессий на пользователя; **rate limiting** на API; запись в audit **факта** открытия (namespace, pod, контейнер); политика «exec в prod» через Velum RBAC. Значения потоков не логировать; при необходимости — опциональная запись сессии только для доверенных ролей и с retention.
- **Impersonation / break-glass:** только для явно помеченных ролей Velum, отдельное событие в audit, максимальная длительность и обоснование (ticket id) — согласовать до реализации.

---

<a id="ref-web-ui-best-practices"></a>

## 🧭 Референсные продукты и лучшие практики Web UI

Ниже — ориентиры из зрелых решений и **что стоит перенять в Velum**, плюс явный список **пробелов** относительно остального документа (фазы / детальный план).

### Таблица референсов (что смотреть и зачем)

| Продукт | Ссылка | Сильные стороны для заимствования |
|--------|--------|-----------------------------------|
| **Headlamp** | [headlamp.dev](https://headlamp.dev/), [архитектура](https://headlamp.dev/docs/latest/development/architecture) | Официальный UI под SIG; **плагины** (Artifact Hub); backend как **прокси** к apiserver; **один WebSocket** из браузера, мультиплексирование на стороне сервера (обход лимита ~6 соединений на origin); UI **отражает RBAC** (недоступные действия скрыты/заблокированы). |
| **Kubernetes Dashboard** | [kubernetes/dashboard](https://github.com/kubernetes/dashboard) | Минималистичный эталон **in-cluster**; вход по **ServiceAccount** / токену; хороший ориентир по **минимальной поверхности атаки**. |
| **Rancher** | [rancher.com](https://www.rancher.com/) | **Мульти-кластер**, проекты/неймспейсы, каталог приложений, политики — для enterprise-сценариев и модели «проект → кластеры». |
| **OpenShift Console** | Документация Red Hat OpenShift | **Operator-centric** UI, **Build/Deployment** связки, шаблоны — для идей по сценариям «приложение», не только сырые ресурсы. |
| **Lens** | [k8slens.dev](https://k8slens.dev/) | **Контексты** kubeconfig, расширения, единый обзор кластера — паттерны навигации и боковых панелей. |
| **Argo CD** (UI) | [argo-cd.readthedocs.io](https://argo-cd.readthedocs.io/) | **Diff** desired vs live, **Sync**, **History/Rollback** для декларативных приложений — переносимо на сравнение манифестов в Velum. |
| **Backstage** (K8s plugin) | [backstage.io](https://backstage.io/) | Связка **каталог сервисов ↔ ресурсы кластера** — если Velum позиционируется как DevOps-платформа. |
| **K9s** | [k9scli.io](https://k9scli.io/) | Не Web, но эталон **горячих клавиш**, фильтров, drill-down — переносимо в shortcuts и плотность таблиц в браузере. |
| **Octant** | [octant.dev](https://octant.dev/) | Проект архивирован; полезен как **источник идей** по навигации и визуализации ресурсов, без ожиданий по живому коду. |

Дополнительно по коду: [kube-rs](https://kube.rs/) (Rust), [client-go](https://github.com/kubernetes/client-go) (эталон поведения informer/watch).

### Лучшие практики управления кластером из Web UI

1. **Идентичность и RBAC:** действия в UI должны выполняться от **той же субъектной модели**, что и у пользователя (OIDC/SA + **impersonation** только для админ-отладки с аудитом). Кнопки и пункты меню **согласованы с `kubectl auth can-i`** — не показывать «успех», который закончится 403 (см. известные UX-ловушки при смешении Role/ClusterRole во вкладках у Headlamp).
2. **Перед записью в etcd:** **server-side dry-run** (`dry-run=server`) — проходит admission, квоты, валидирующие webhooks; отдельно от «клиентской» проверки YAML.
3. **Server-Side Apply (SSA):** для редактора YAML — учёт **конфликтов полей** (`managedFields`), опция «force» только с явным предупреждением; см. [документацию SSA](https://kubernetes.io/docs/reference/using-api/server-side-apply/).
4. **Поток «diff → apply»:** показ изменений до применения (как GitOps-UI); для Helm — уже частично в плане Фазы 9.
5. **WebSocket:** один канал на сессию + **мультиплексирование** подписок на ресурсы/namespace (как Headlamp), бэкпрешур при перегрузке apiserver.
6. **Пагинация и `continue`:** списки больших ресурсов только с **лимитом и продолжением**; не тянуть «все поды кластера» без фильтра.
7. **Наблюдаемость в UI:** колонка **Conditions**, ссылки на **Events**, для workload — **ReplicaSet/Deployment связь**; для отладки — **ephemeral containers** (если политика PSA позволяет).
8. **Секреты:** маскирование по умолчанию, копирование по явному действию, без логирования значений.
9. **Плагины / расширяемость:** длинный хвост **CRD** через плагины или динамические формы по OpenAPI schema (в плане есть CRD — усилить темой плагинов).
10. **Доступность и операционный комфорт:** тёмная тема, клавиатурная навигация (наследие K9s), понятные **таймауты** и сообщения при **429/503** (APF на apiserver).

### Чего не хватает в нижестоящем плане (добавить в backlog)

Следующие пункты **не выделены отдельными фазами** выше; рекомендуется вынести в backlog после MVP или встроить в Фазу 1–2 как NFR.

| Тема | Описание |
|------|----------|
| **Server-side dry-run + diff** | Явные эндпоинты/шаги мастера перед Apply; сравнение с текущим объектом в etcd. |
| **SSA и конфликты полей** | Отображение конфликта SSA; стратегия merge vs replace. |
| **Проверка прав в UI** | Вызов `SelfSubjectRulesReview` / кэш `can-i` по виду ресурса и namespace. |
| **Архитектура WebSocket** | Один WS, мультиплексирование; лимиты подписок (см. KPI). |
| **Плагины / marketplace** | Модель как у Headlamp (опционально, post-1.0). |
| **Impersonation** | Только для доверенных ролей, аудит, согласование с Velum RBAC. |
| **Сервис-меш / ingress-NGINX** | Опциональные экраны (Istio/Linkerd/NGINX) — не в текущих фазах. |
| **FinOps** | Интеграция с Kubecost / OpenCost для стоимости по namespace — опционально. |
| **Копировать как `kubectl`** | Генерация эквивалентной команды для обучения и runbooks. |
| **Объяснение полей** | Встроенный «explain» по полям ресурса (аналог `kubectl explain`). |

Эти строки дополняют разделы [Фазы реализации](#фазы-реализации), [KPI](#kpi-и-нефункциональные-требования) и [Дополнительные ресурсы](#дополнительные-ресурсы-outline), но **не заменяют** их: при приоритизации закладывать время на «безопасный apply» и **честный RBAC-UX** не меньше, чем на CRUD по ресурсам.

---

## 📅 Фазы реализации

Оценка в **неделях** ориентировочна для **одного** полного потока разработки; при меньшей команде умножить или сузить scope MVP.

### 🔹 Фаза 1: Фундамент (Недели 1-2)
**Цель:** Базовое подключение и чтение ресурсов

- [ ] PoC `kube::Client` + list namespaces (параллельно существующему [kubernetes/client.rs](rust/src/kubernetes/client.rs))
- [ ] Базовая инфраструктура Kubernetes клиента
- [ ] Подключение к кластеру (kubeconfig, in-cluster)
- [ ] **Задел multi-cluster:** модель «одно подключение за запрос» (cluster id в контексте сессии/API), хранение метаданных подключений — без полноценного UI переключателя (он в фазе 10)
- [ ] Health checks и валидация подключения
- [ ] Список namespace'ов
- [ ] Базовая авторизация (RBAC sync)
- [ ] Задел под **RBAC-UX:** как минимум проверка прав на ключевые verbs перед отображением действий (см. [лучшие практики Web UI](#ref-web-ui-best-practices))

**Definition of Done:**
- ✅ Health endpoint возвращает статус кластера
- ✅ Список namespace'ов загружается < 500ms
- ✅ Ошибки подключения обрабатываются gracefully

---

### 🔹 Фаза 2: Core Workloads (Недели 3-5)
**Цель:** Полный CRUD для основных workload ресурсов

- [ ] **Events (минимум для workloads):** list/get по namespace и по involved object (pod/deployment) — см. раздел [16. Events](#16-events); полноценный cluster-wide стрим и топология — в [фазе 8](#фазы-реализации)
- [ ] Pods (CRUD, логи, exec, port-forward)
- [ ] Редактор YAML / apply с **server-side dry-run** и по возможности **diff** к live-объекту (backlog деталей — таблица в разделе референсов)
- [ ] Deployments (scale, rollout, rollback)
- [ ] ReplicaSets
- [ ] DaemonSets
- [ ] StatefulSets

**Definition of Done:**
- ✅ Pod logs streaming через WebSocket
- ✅ Exec terminal работает стабильно
- ✅ Deployment scale/restart/rollback работают
- ✅ YAML editor с валидацией schema

---

### 🔹 Фаза 3: Networking & Config (Недели 6-7)
**Цель:** Сетевые ресурсы и конфигурация

- [ ] Services (ClusterIP, NodePort, LoadBalancer)
- [ ] ConfigMaps
- [ ] Secrets (маскирование в UI; **encryption at rest** — зона etcd/KMS кластера, не отдельный «encrypt API» Velum)
- [ ] Ingress & IngressClass (**API group:** `networking.k8s.io`, не `extensions/v1beta1`)
- [ ] NetworkPolicy
- [ ] (Опционально, после Ingress) **Gateway API** — `Gateway`, `HTTPRoute`, `GRPCRoute` там, где кластер использует Gateway API вместо классического Ingress

**Definition of Done:**
- ✅ Отображение backend'ов сервиса: **EndpointSlices** (предпочтительно) и/или legacy Endpoints, если включено в кластере
- ✅ Secrets с mask/unmask values
- ✅ Ingress routing rules diagram

---

### 🔹 Фаза 4: Storage (Неделя 8)
**Цель:** Управление persistent storage

- [ ] PersistentVolumes
- [ ] PersistentVolumeClaims
- [ ] StorageClass
- [ ] VolumeSnapshots

**Definition of Done:**
- ✅ PV/PVC binding visualization
- ✅ StorageClass provisioning отображение

---

### 🔹 Фаза 5: Batch & Scheduling (Неделя 9)
**Цель:** Batch workload и scheduling

- [ ] Jobs
- [ ] CronJobs
- [ ] PriorityClass
- [ ] PodDisruptionBudget

**Definition of Done:**
- ✅ **CronJob** suspend/resume (`spec.suspend`); **Job** — без suspend/resume на уровне API (отличие от CronJob)
- ✅ Job history timeline
- ✅ Next schedule calculation

---

### 🔹 Фаза 6: RBAC & Security (Неделя 10)
**Цель:** Полное управление доступом

- [ ] ServiceAccounts
- [ ] Roles & RoleBindings
- [ ] ClusterRoles & ClusterRoleBindings
- [ ] **Pod Security Admission (PSA)** — уровни `privileged` / `baseline` / `restricted` через labels на Namespace; отображение и подсказки в UI. **PodSecurityPolicy (PSP) не использовать** — удалены из Kubernetes 1.25+

**Definition of Done:**
- ✅ RBAC matrix visualization
- ✅ SelfSubjectRulesReview integration
- ✅ PSA labels editor

---

### 🔹 Фаза 7: Advanced (Недели 11-12)
**Цель:** Расширенные ресурсы

- [ ] CustomResourceDefinitions
- [ ] Operators (базовая поддержка)
- [ ] HorizontalPodAutoscaler
- [ ] VerticalPodAutoscaler
- [ ] LimitRange & ResourceQuota

**Definition of Done:**
- ✅ CRD dynamic form по OpenAPI schema
- ✅ HPA metrics отображение
- ✅ ResourceQuota usage charts

---

### 🔹 Фаза 8: Observability (Недели 13-14)
**Цель:** Наблюдаемость и мониторинг

- [ ] Metrics API integration
- [ ] **Cluster-wide Events stream** и агрегированные представления (на базе событий из [фазы 2](#фазы-реализации))
- [ ] Logs aggregation
- [ ] Topology visualization
- [ ] Resource usage charts

**Definition of Done:**
- ✅ Events WebSocket stream < 100ms latency
- ✅ Topology map с Cytoscape.js
- ✅ Historical metrics charts

---

### 🔹 Фаза 9: Helm Integration (Неделя 15)
**Цель:** Управление Helm чартами (в коде уже есть [rust/src/kubernetes/helm.rs](rust/src/kubernetes/helm.rs) — фаза про **HTTP API + UI**, а не с нуля про бизнес-логику CLI)

- [ ] Helm releases
- [ ] Chart catalog
- [ ] Install/Upgrade/Rollback
- [ ] Repository management

**Definition of Done:**
- ✅ Chart install wizard
- ✅ Release history с rollback
- ✅ Values editor с validation

---

### 🔹 Фаза 10: Polish & Enterprise (Недели 16-18)
**Цель:** Production-ready продукт

- [ ] Multi-cluster management
- [ ] Audit logging
- [ ] Backup/Restore
- [ ] GitOps integration
- [ ] AI-assistant (опционально)
- [ ] **Server-side dry-run + diff** для всех apply операций
- [ ] **SSA конфликты полей** с UI предупреждениями
- [ ] **kubectl command generator** — показывать эквивалентную команду для обучения

**Definition of Done:**
- ✅ Переключение между кластерами
- ✅ Audit log export
- ✅ Dark/Light theme
- ✅ Mobile responsive
- ✅ i18n (EN/RU)
- ✅ Accessibility WCAG 2.1 AA

---

## 📊 Детальный план по ресурсам

### 1. Namespaces

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/namespaces.rs
GET    /api/kubernetes/namespaces              // Список всех namespace
GET    /api/kubernetes/namespaces/{name}       // Детали namespace
POST   /api/kubernetes/namespaces              // Создать namespace
PUT    /api/kubernetes/namespaces/{name}       // Обновить namespace
DELETE /api/kubernetes/namespaces/{name}       // Удалить namespace
GET    /api/kubernetes/namespaces/{name}/quota // ResourceQuota
GET    /api/kubernetes/namespaces/{name}/limits// LimitRange
```

**Frontend:**
- Список namespace'ов с метриками (CPU/Memory usage)
- Создание через модальное окно
- Квоты и лимиты в виде карточек

---

### 2. Pods

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/pods.rs
GET    /api/kubernetes/pods                    // Список pod'ов (все namespace)
GET    /api/kubernetes/namespaces/{ns}/pods    // Список pod'ов в namespace
GET    /api/kubernetes/namespaces/{ns}/pods/{name}        // Детали pod
DELETE /api/kubernetes/namespaces/{ns}/pods/{name}        // Удалить pod
GET    /api/kubernetes/namespaces/{ns}/pods/{name}/logs   // Логи
POST   /api/kubernetes/namespaces/{ns}/pods/{name}/exec   // Exec terminal
POST   /api/kubernetes/namespaces/{ns}/pods/{name}/port-forward // Port-forward
GET    /api/kubernetes/namespaces/{ns}/pods/{name}/yaml   // YAML manifest
PUT    /api/kubernetes/namespaces/{ns}/pods/{name}/yaml   // Обновить YAML
POST   /api/kubernetes/namespaces/{ns}/pods/{name}/evict  // Evict pod
```

**Frontend:**
- Таблица pod'ов со статусами (цветные бейджи)
- Быстрые действия: view logs, exec, delete
- Детальная страница с:
  - Контейнеры и их статусы
  - Volumes mounts
  - Environment variables
  - Events pod'а
  - Графики CPU/Memory
- YAML редактор с подсветкой
- Встроенный terminal (WebSocket)

---

### 3. Deployments

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/deployments.rs
GET    /api/kubernetes/deployments                        // Список
GET    /api/kubernetes/namespaces/{ns}/deployments/{name} // Детали
POST   /api/kubernetes/deployments                        // Создать
PUT    /api/kubernetes/namespaces/{ns}/deployments/{name} // Обновить
DELETE /api/kubernetes/namespaces/{ns}/deployments/{name} // Удалить
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/scale  // Scale
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/restart // Restart
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/pause   // Pause rollout
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/resume  // Resume rollout
POST   /api/kubernetes/namespaces/{ns}/deployments/{name}/rollback // Rollback
GET    /api/kubernetes/namespaces/{ns}/deployments/{name}/history // Rollout history
GET    /api/kubernetes/namespaces/{ns}/deployments/{name}/replicasets // Linked ReplicaSets
```

**Frontend:**
- Список deployment'ов с репликами (желательно/доступно/готово)
- Кнопки: Scale (+/-), Restart, Rollback
- Визуализация rollout status
- История ревизий с возможностью отката
- Связанные ReplicaSets
- YAML editor

---

### 4. ReplicaSets

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/replicasets.rs
GET    /api/kubernetes/replicasets
GET    /api/kubernetes/namespaces/{ns}/replicasets/{name}
DELETE /api/kubernetes/namespaces/{ns}/replicasets/{name}
GET    /api/kubernetes/namespaces/{ns}/replicasets/{name}/pods // Linked Pods
```

---

### 5. DaemonSets

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/daemonsets.rs
GET    /api/kubernetes/daemonsets
GET    /api/kubernetes/namespaces/{ns}/daemonsets/{name}
POST   /api/kubernetes/daemonsets
PUT    /api/kubernetes/namespaces/{ns}/daemonsets/{name}
DELETE /api/kubernetes/namespaces/{ns}/daemonsets/{name}
GET    /api/kubernetes/namespaces/{ns}/daemonsets/{name}/pods
```

**Frontend:**
- Статистика: Nodes selected / Pods running
- Список pod'ов на каждой ноде

---

### 6. StatefulSets

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/statefulsets.rs
GET    /api/kubernetes/statefulsets
GET    /api/kubernetes/namespaces/{ns}/statefulsets/{name}
POST   /api/kubernetes/statefulsets
PUT    /api/kubernetes/namespaces/{ns}/statefulsets/{name}
DELETE /api/kubernetes/namespaces/{ns}/statefulsets/{name}
POST   /api/kubernetes/namespaces/{ns}/statefulsets/{name}/scale
```

**Frontend:**
- Отображение порядковых pod'ов (pod-0, pod-1, ...)
- Связанные PersistentVolumeClaims
- Headless Service

---

### 7. Services

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/services.rs
GET    /api/kubernetes/services
GET    /api/kubernetes/namespaces/{ns}/services/{name}
POST   /api/kubernetes/services
PUT    /api/kubernetes/namespaces/{ns}/services/{name}
DELETE /api/kubernetes/namespaces/{ns}/services/{name}
GET    /api/kubernetes/namespaces/{ns}/services/{name}/endpoints // Legacy Endpoints (если нужны)
GET    /api/kubernetes/namespaces/{ns}/services/{name}/endpoint-slices // EndpointSlices (предпочтительно, discovery.k8s.io)
```

**Frontend:**
- Тип сервиса (ClusterIP/NodePort/LoadBalancer) бейджом
- Cluster IP, External IP, Ports
- Backend'ы: **EndpointSlices** (основной ориентир); **Endpoints** — при необходимости совместимости
- Selector matching

---

### 8. ConfigMaps

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/configmaps.rs
GET    /api/kubernetes/configmaps
GET    /api/kubernetes/namespaces/{ns}/configmaps/{name}
POST   /api/kubernetes/configmaps
PUT    /api/kubernetes/namespaces/{ns}/configmaps/{name}
DELETE /api/kubernetes/namespaces/{ns}/configmaps/{name}
GET    /api/kubernetes/namespaces/{ns}/configmaps/{name}/yaml
```

**Frontend:**
- Список key-value пар
- Редактор данных (text/yaml)
- Где используется (referenced by)

---

### 9. Secrets

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/secrets.rs
GET    /api/kubernetes/secrets
GET    /api/kubernetes/namespaces/{ns}/secrets/{name}
POST   /api/kubernetes/secrets
PUT    /api/kubernetes/namespaces/{ns}/secrets/{name}
DELETE /api/kubernetes/namespaces/{ns}/secrets/{name}
// Примечание: «шифрование at rest» — настройка etcd/KMS кластера, не отдельный POST в Velum.
```

**Frontend:**
- Типы: Opaque, docker-registry, TLS, Basic Auth
- Base64 декодирование/кодирование
- Masked values по умолчанию
- Интеграция с External Secrets (опционально)

---

### 10. Ingress

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/ingress.rs
GET    /api/kubernetes/ingress
GET    /api/kubernetes/namespaces/{ns}/ingress/{name}
POST   /api/kubernetes/ingress
PUT    /api/kubernetes/namespaces/{ns}/ingress/{name}
DELETE /api/kubernetes/namespaces/{ns}/ingress/{name}
GET    /api/kubernetes/ingress-classes // IngressClass list
```

**API:** ресурсы `Ingress` и `IngressClass` — group **`networking.k8s.io`** (стабильные версии согласно целевой версии кластера; не использовать удалённый `extensions/v1beta1`).

**Frontend:**
- Rules table: Host | Path | Service | Port
- TLS секция
- Annotations
- Визуализация routing rules

---

### 11. NetworkPolicy

**Backend API:**
```rust
GET    /api/kubernetes/network-policies
GET    /api/kubernetes/namespaces/{ns}/network-policies/{name}
POST   /api/kubernetes/network-policies
PUT    /api/kubernetes/namespaces/{ns}/network-policies/{name}
DELETE /api/kubernetes/namespaces/{ns}/network-policies/{name}
```

**Frontend:**
- Ingress/Egress rules визуализация
- Pod selector
- Policy types

---

### 12. PersistentVolumes & PVC

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/storage.rs
GET    /api/kubernetes/persistent-volumes      // Cluster-scoped
GET    /api/kubernetes/persistent-volumes/{name}
POST   /api/kubernetes/persistent-volumes
DELETE /api/kubernetes/persistent-volumes/{name}

GET    /api/kubernetes/persistent-volume-claims
GET    /api/kubernetes/namespaces/{ns}/pvc/{name}
POST   /api/kubernetes/persistent-volume-claims
DELETE /api/kubernetes/namespaces/{ns}/pvc/{name}

GET    /api/kubernetes/storage-classes
POST   /api/kubernetes/storage-classes
DELETE /api/kubernetes/storage-classes/{name}
```

**Frontend:**
- PV: Capacity, Access Modes, Status, Claim
- PVC: Status, Volume, StorageClass
- StorageClass: Provisioner, Reclaim Policy

---

### 13. Jobs & CronJobs

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/batch.rs
GET    /api/kubernetes/jobs
GET    /api/kubernetes/namespaces/{ns}/jobs/{name}
POST   /api/kubernetes/jobs
DELETE /api/kubernetes/namespaces/{ns}/jobs/{name}

GET    /api/kubernetes/cronjobs
GET    /api/kubernetes/namespaces/{ns}/cronjobs/{name}
POST   /api/kubernetes/cronjobs
PUT    /api/kubernetes/namespaces/{ns}/cronjobs/{name}
DELETE /api/kubernetes/namespaces/{ns}/cronjobs/{name}
POST   /api/kubernetes/namespaces/{ns}/cronjobs/{name}/suspend
POST   /api/kubernetes/namespaces/{ns}/cronjobs/{name}/resume
GET    /api/kubernetes/namespaces/{ns}/cronjobs/{name}/history // Linked Jobs
```

**Frontend:**
- Jobs: Completions, Duration, Status
- CronJobs: Schedule, Last Schedule, Suspend toggle
- История запусков

---

### 14. RBAC

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/rbac.rs
// ServiceAccounts
GET    /api/kubernetes/service-accounts
GET    /api/kubernetes/namespaces/{ns}/service-accounts/{name}
POST   /api/kubernetes/service-accounts
DELETE /api/kubernetes/namespaces/{ns}/service-accounts/{name}

// Roles & RoleBindings
GET    /api/kubernetes/roles
GET    /api/kubernetes/namespaces/{ns}/roles/{name}
POST   /api/kubernetes/roles
DELETE /api/kubernetes/namespaces/{ns}/roles/{name}

GET    /api/kubernetes/role-bindings
GET    /api/kubernetes/namespaces/{ns}/role-bindings/{name}
POST   /api/kubernetes/role-bindings
DELETE /api/kubernetes/namespaces/{ns}/role-bindings/{name}

// ClusterRoles & ClusterRoleBindings
GET    /api/kubernetes/cluster-roles
GET    /api/kubernetes/cluster-roles/{name}
POST   /api/kubernetes/cluster-roles
DELETE /api/kubernetes/cluster-roles/{name}

GET    /api/kubernetes/cluster-role-bindings
GET    /api/kubernetes/cluster-role-bindings/{name}
POST   /api/kubernetes/cluster-role-bindings
DELETE /api/kubernetes/cluster-role-bindings/{name}
```

**Frontend:**
- ServiceAccounts с linked secrets
- Roles: Verbs + Resources матрица
- Bindings: Subject + RoleRef

---

### 15. HPA & VPA

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/autoscaling.rs
GET    /api/kubernetes/horizontal-pod-autoscalers
GET    /api/kubernetes/namespaces/{ns}/hpa/{name}
POST   /api/kubernetes/horizontal-pod-autoscalers
PUT    /api/kubernetes/namespaces/{ns}/hpa/{name}
DELETE /api/kubernetes/namespaces/{ns}/hpa/{name}
```

**Frontend:**
- Min/Max replicas
- Current/Target metrics
- Scale target reference

---

### 16. Events

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/events.rs
GET    /api/kubernetes/events                    // Все события
GET    /api/kubernetes/namespaces/{ns}/events    // События namespace
GET    /api/kubernetes/namespaces/{ns}/events/{name} // Детали
WS     /api/kubernetes/namespaces/{ns}/events/stream // WebSocket stream
```

**Frontend:**
- Real-time stream через WebSocket
- Фильтры: Type (Normal/Warning), Involved Object
- Timeline view

---

### 17. Metrics

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/metrics.rs
GET    /api/kubernetes/metrics/nodes             // Node metrics
GET    /api/kubernetes/metrics/pods              // Pod metrics
GET    /api/kubernetes/namespaces/{ns}/metrics/pods/{name}
```

**Frontend:**
- CPU/Memory usage charts
- Top nodes, Top pods
- Historical graphs

---

### 18. CustomResourceDefinitions

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/crd.rs
GET    /api/kubernetes/custom-resources          // Список CRD
GET    /api/kubernetes/custom-resources/{group}/{version}/{plural}
POST   /api/kubernetes/custom-resources/{group}/{version}/{plural}
PUT    /api/kubernetes/custom-resources/{group}/{version}/{plural}/{name}
DELETE /api/kubernetes/custom-resources/{group}/{version}/{plural}/{name}
```

**Frontend:**
- Список CRD
- Dynamic form для CR (на основе OpenAPI schema)
- YAML editor

---

### 19. Cluster Overview

**Backend API:**
```rust
// rust/src/api/handlers/kubernetes/cluster.rs
GET    /api/kubernetes/cluster/info              // Cluster info
GET    /api/kubernetes/cluster/nodes             // Nodes list
GET    /api/kubernetes/cluster/nodes/{name}      // Node details
GET    /api/kubernetes/cluster/health            // Health status
GET    /api/kubernetes/cluster/resources         // Resource summary
GET    /api/kubernetes/cluster/version           // Kubernetes version
```

**Frontend:**
- Cluster health dashboard
- Nodes grid с метриками
- Resource usage pie charts
- Kubernetes version badge

---

### 20. Дополнительные ресурсы (outline)

Ниже — ресурсы и области API, не раскрытые отдельными подразделами выше; для MVP достаточно read-only list/get, затем CRUD по приоритету.

| Область | Ресурсы (ориентир) |
|--------|---------------------|
| Cluster | **Nodes**, Leases (опц.), **RuntimeClass**, Namespace (уже в плане) |
| Discovery | **APIResources** (kubectl `api-resources`), **APIService**, **EndpointSlice** (`discovery.k8s.io`) — см. также Services |
| Workloads (legacy) | **ReplicationController** (наследие), ReplicaSet (связан с Deployment) |
| Policy / quota | **ResourceQuota**, **LimitRange**, **ValidatingAdmissionPolicy** / **ValidatingAdmissionPolicyBinding** (1.26+), NetworkPolicy (уже в плане); опционально UI/ссылки на политики **Kyverno / OPA Gatekeeper** (вне core API) |
| Storage / CSI | **CSIDriver**, **CSINode**, **VolumeAttachment** (операторский уровень; рядом с PV/PVC/StorageClass) |

---

## API Reference

Контракт REST для целевого UI: префикс **`/api/kubernetes/...`** — эндпоинты перечислены в разделе [Детальный план по ресурсам](#детальный-план-по-ресурсам) (под каждым ресурсом). Единый машиночитаемый контракт: файл **`api/kubernetes-openapi.yml`** (или генерация из Rust — на усмотрение реализации); публикация в Swagger UI опциональна (`/api/docs`).

### Соглашения по URL (единый стиль)

При реализации **выровнять** фактические пути с OpenAPI (ниже — целевое правило; отдельные примеры в детальном плане могут отличаться до рефакторинга контракта):

- Имена коллекций: **множественное число**, `kebab-case`: `namespaces`, `pods`, `deployments`, `ingresses`, `network-policies`, `persistent-volume-claims`.
- Namespace-scoped: `/api/kubernetes/namespaces/{ns}/<collection>/{name}`.
- Cluster-scoped: `/api/kubernetes/<collection>/{name}`.
- Подресурсы: суффикс `/logs`, `/exec`, `/scale`, `/yaml` и т.д. — как в OpenAPI.

---

## 🎨 Frontend компоненты

### Согласование с Velum (`web/public`)

Новый K8s UI должен **вписываться в существующий фронт**: те же шрифты (Roboto), боковая панель (#005057), паттерны из `app.js` (навигация, `api` client).

**Решение по умолчанию:** страницы и скрипты под **[web/public/k8s/](web/public/k8s/)** (`k8s-*.html` или подкаталог `pages/`), пункт «Kubernetes» в общем sidebar. Отклонение от этого — только осознанно (единый `styles.css` обязателен).

### Структура файлов
```
web/public/k8s/
│   ├── k8s.js                 # Kubernetes API client
│   ├── k8s-websocket.js       # WebSocket subscriptions
│   ├── components/
│   │   ├── cluster-overview.js
│   │   ├── namespace-picker.js
│   │   ├── resource-table.js
│   │   ├── yaml-editor.js
│   │   ├── pod-details.js
│   │   ├── deployment-scaler.js
│   │   ├── terminal-exec.js
│   │   ├── logs-viewer.js
│   │   ├── topology-map.js
│   │   └── metrics-charts.js
│   ├── pages/
│   │   ├── k8s-dashboard.html
│   │   ├── k8s-pods.html
│   │   ├── k8s-deployments.html
│   │   ├── k8s-services.html
│   │   ├── k8s-configmaps.html
│   │   ├── k8s-secrets.html
│   │   ├── k8s-ingress.html
│   │   ├── k8s-storage.html
│   │   ├── k8s-jobs.html
│   │   ├── k8s-rbac.html
│   │   ├── k8s-crd.html
│   │   └── k8s-cluster.html
│   └── styles/
│       └── kubernetes.css
```

(Каталог `pages/` при необходимости на том же уровне, что и `components/`.)

### k8s.js — API Client
```javascript
// Базовый клиент для Kubernetes API
const k8s = {
  // Namespaces
  async listNamespaces() {
    return api.get('/api/kubernetes/namespaces');
  },

  // Pods
  async listPods(namespace) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/pods`);
  },

  async getPod(namespace, name) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/pods/${name}`);
  },

  async deletePod(namespace, name) {
    return api.delete(`/api/kubernetes/namespaces/${namespace}/pods/${name}`);
  },

  async getPodLogs(namespace, name, container) {
    const params = container ? `?container=${container}` : '';
    return api.get(`/api/kubernetes/namespaces/${namespace}/pods/${name}/logs${params}`);
  },

  // Deployments
  async listDeployments(namespace) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/deployments`);
  },

  async scaleDeployment(namespace, name, replicas) {
    return api.post(`/api/kubernetes/namespaces/${namespace}/deployments/${name}/scale`, { replicas });
  },

  async restartDeployment(namespace, name) {
    return api.post(`/api/kubernetes/namespaces/${namespace}/deployments/${name}/restart`);
  },

  async rollbackDeployment(namespace, name, revision) {
    return api.post(`/api/kubernetes/namespaces/${namespace}/deployments/${name}/rollback`, { revision });
  },

  // Services
  async listServices(namespace) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/services`);
  },

  // ConfigMaps
  async listConfigMaps(namespace) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/configmaps`);
  },

  // Secrets
  async listSecrets(namespace) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/secrets`);
  },

  // Metrics
  async getPodMetrics(namespace, name) {
    return api.get(`/api/kubernetes/namespaces/${namespace}/pods/${name}/metrics`);
  },

  async getNodeMetrics() {
    return api.get('/api/kubernetes/metrics/nodes');
  },

  // Cluster
  async getClusterInfo() {
    return api.get('/api/kubernetes/cluster/info');
  },

  async getClusterNodes() {
    return api.get('/api/kubernetes/cluster/nodes');
  },
};
```

### WebSocket subscriptions
```javascript
// k8s-websocket.js
class KubernetesWebSocket {
  constructor() {
    this.ws = null;
    this.subscriptions = new Map();
  }

  connect() {
    this.ws = new WebSocket(`${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/api/kubernetes/ws`);

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      const { type, namespace, resource, name, data } = message;

      // Notify subscribers
      const key = `${namespace}/${resource}/${name}`;
      const subscribers = this.subscriptions.get(key) || [];
      subscribers.forEach(cb => cb(data));
    };
  }

  subscribe(namespace, resource, name, callback) {
    const key = `${namespace}/${resource}/${name}`;
    if (!this.subscriptions.has(key)) {
      this.subscriptions.set(key, []);
      this.ws.send(JSON.stringify({
        action: 'subscribe',
        namespace,
        resource,
        name
      }));
    }
    this.subscriptions.get(key).push(callback);
  }

  unsubscribe(namespace, resource, name, callback) {
    const key = `${namespace}/${resource}/${name}`;
    const subscribers = this.subscriptions.get(key) || [];
    const index = subscribers.indexOf(callback);
    if (index > -1) {
      subscribers.splice(index, 1);
    }
  }
}

const k8sWs = new KubernetesWebSocket();
```

---

## 🔌 Интеграции

### Helm
```rust
// rust/src/api/handlers/kubernetes/helm.rs
GET    /api/kubernetes/helm/repos                // Helm repositories
GET    /api/kubernetes/helm/charts               // Available charts
POST   /api/kubernetes/helm/install              // Install chart
PUT    /api/kubernetes/helm/releases/{name}/upgrade // Upgrade release
POST   /api/kubernetes/helm/releases/{name}/rollback // Rollback release
DELETE /api/kubernetes/helm/releases/{name}      // Uninstall release
GET    /api/kubernetes/helm/releases             // List releases
```

### GitOps (ArgoCD/Flux)
```rust
GET    /api/kubernetes/gitops/applications       // ArgoCD applications
POST   /api/kubernetes/gitops/sync               // Sync application
```

### Metrics Server
```rust
GET    /api/kubernetes/metrics/nodes
GET    /api/kubernetes/metrics/pods
```

### Prometheus (опционально)
```rust
GET    /api/kubernetes/prometheus/query          // PromQL query
GET    /api/kubernetes/prometheus/range_query    // Range query for charts
```

---

## 🧪 Тестирование

### CI и кластер для интеграции

- Локально / в CI: **`kind`** или **minikube**; kubeconfig в GitHub Actions — через **secrets** (base64 или отдельный step setup-kind).
- Контракт backend ↔ frontend: держать **OpenAPI** (`api/kubernetes-openapi.yml`) в синхроне с реализацией или генерировать из кода.

### Unit тесты (Rust)
```rust
// rust/tests/kubernetes_api_test.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_pods() {
        let client = create_test_client();
        let pods = client.list_pods("default").await.unwrap();
        assert!(pods.iter().any(|p| p.name == "test-pod"));
    }

    #[tokio::test]
    async fn test_scale_deployment() {
        let client = create_test_client();
        let result = client.scale_deployment("default", "test-deploy", 3).await;
        assert!(result.is_ok());
    }
}
```

### Integration тесты
```bash
# test/kubernetes/test-k8s-api.sh
#!/bin/bash
set -e

# Базовый URL: локально по умолчанию Velum слушает :3000 (см. rust/src/cli/cmd_server.rs); Docker demo — часто :8088.
BASE_URL="${VELUM_BASE_URL:-http://localhost:3000}"

echo "Testing Kubernetes API against ${BASE_URL}..."

curl -s "${BASE_URL}/api/kubernetes/namespaces" | jq '. | length > 0'
curl -s "${BASE_URL}/api/kubernetes/namespaces/default/pods" | jq '.'
curl -s "${BASE_URL}/api/kubernetes/cluster/info" | jq '.kubernetes_version'

echo "All Kubernetes API tests passed!"
```

### E2E тесты

**Опционально — Playwright + TypeScript** (ниже пример). Основной стек репозитория — **Vanilla JS**; альтернатива — headless-проверки через **gstack/browse** (репозиторий `.claude/skills/gstack/browse`) или минимальный Playwright только для smoke.

#### Пример (Playwright, опционально)
```typescript
// test/kubernetes/e2e/kubernetes.spec.ts
import { test, expect } from '@playwright/test';

test.describe('Kubernetes Dashboard', () => {
  test('should display cluster overview', async ({ page }) => {
    await page.goto('/kubernetes/cluster');
    await expect(page.locator('#nodes-grid')).toBeVisible();
    await expect(page.locator('.node-card')).toHaveCount.greaterThan(0);
  });

  test('should list pods and show details', async ({ page }) => {
    await page.goto('/kubernetes/pods');
    await page.click('.pod-row:first-child');
    await expect(page.locator('#pod-details')).toBeVisible();
    await expect(page.locator('#pod-logs')).toBeVisible();
  });

  test('should scale deployment', async ({ page }) => {
    await page.goto('/kubernetes/deployments');
    await page.click('[data-testid="scale-btn"]');
    await page.fill('#replicas-input', '5');
    await page.click('#confirm-scale');
    await expect(page.locator('.replicas-status')).toContainText('5/5');
  });
});
```

---

## 📚 Документация

### Типы документации

1. **User Guide** — для конечных пользователей
2. **Admin Guide** — для администраторов платформы
3. **Developer Guide** — для контрибьюторов
4. **API Reference** — авто-генерация из кода (OpenAPI/Swagger)
5. **Runbooks** — для operacional support

### Структура документации

```
docs/kubernetes/
├── getting-started/
│   ├── installation.md
│   ├── configuration.md
│   └── quickstart.md
├── user-guide/
│   ├── cluster-management.md
│   ├── workloads.md
│   ├── networking.md
│   ├── storage.md
│   ├── rbac.md
│   └── helm.md
├── admin-guide/
│   ├── authentication.md
│   ├── multi-cluster.md
│   ├── audit-logging.md
│   └── backup-restore.md
├── developer-guide/
│   ├── architecture.md
│   ├── contributing.md
│   ├── api-guidelines.md
│   └── testing.md
├── runbooks/
│   ├── troubleshooting.md
│   ├── performance-tuning.md
│   └── security-hardening.md
└── api-reference/
    └── openapi.yml (авто-генерация)
```

### Auto-generated API Docs

Использовать `utoipa` или `paperclip` для генерации OpenAPI spec из Rust кода:

```rust
// В handlers добавить #[openapi] атрибуты
#[openapi(
    summary = "List all namespaces",
    tag = "Kubernetes",
    responses(
        (status = 200, description = "List of namespaces", body = Vec<NamespaceSummary>),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal error")
    )
)]
pub async fn list_namespaces(...) { ... }
```

### Запуск Velum для разработки K8s UI

```bash
cd rust
# HTTP по умолчанию 0.0.0.0:3000; порт: -p / --port
cargo run -- server
```

---

## 🎯 KPI и нефункциональные требования

Целевые цифры **зависят от apiserver, сети и размера кластера** — фиксировать как SLO отдельно для read-only и mutating.

- **Покрытие возможностей kubectl через UI** — поэтапно; полный паритет — долгосрочная цель, не блокер первого релиза модуля.
- **Латентность read-only API** — низкая на тёплом кэше; большие списки — пагинация, `limit`/`continue`, таймауты.
- **Латентность mutating** — отдельный бюджет (admission, reconcile).
- **Загрузка страниц** — ориентир комфорта UX; тяжёлые таблицы — виртуализация.
- **Real-time** — задержка watch/WebSocket с учётом нагрузки; лимиты на число **watch** и размер **informer/cache**; учитывать **429/503** и APF на apiserver.
- **Доступность:** **WCAG 2.1 AA** и **i18n (EN/RU)** — закрываются в [фазе 10](#фазы-реализации) (Definition of Done), не откладывать «без фазы».
- **Mobile responsive** — планшеты и узкие экраны (минимум чтение и критичные действия).

---

## ⚠️ Риски и митигация

| Риск | Вероятность | Влияние | Митигация |
|------|-------------|---------|-----------|
| **WebSocket лимиты браузера** (~6 соединений на origin) | Высокая | Высокое | Мультиплексирование через один WS канал (как Headlamp) |
| **RBAC сложности** (Role vs ClusterRole) | Высокая | Среднее | Явная проверка `SelfSubjectRulesReview` перед отображением UI элементов |
| **SSA конфликты полей** | Средняя | Высокое | Dry-run + diff перед apply, явные предупреждения о конфликтах |
| **Performance при больших кластерах** | Средняя | Высокое | Пагинация, limit/watch, кэширование через informer |
| **Security (credentials leakage)** | Низкая | Критичное | Шифрование kubeconfig, краткосрочные токены, audit logging |
| **API version incompatibility** | Средняя | Среднее | Поддержка нескольких версий через `k8s-openapi` features |
| **Exec/port-forward security** | Средняя | Высокое | Явная авторизация, timeout, audit, rate limiting |

---

## 🔧 Инструменты разработки

### Local Development

```bash
# Minikube для тестового кластера
minikube start --kubernetes-version=v1.30.0

# Kind для multi-cluster тестов
kind create cluster --name test-1
kind create cluster --name test-2

# kubeconfig merge
kind export kubeconfig --name test-1 --kubeconfig ~/.kube/config
kind export kubeconfig --name test-2 --kubeconfig ~/.kube/config
```

### Testing

```bash
# Unit tests
cargo test --package velum -- kubernetes::

# Integration tests с test cluster
cargo test --test kubernetes_integration

# E2E tests
npm run test:e2e -- kubernetes
```

### Debugging

```bash
# Watch kubectl events
kubectl get events --watch

# Trace API calls
kubectl get --raw /apis --v=8

# Check RBAC
kubectl auth can-i get pods --as system:serviceaccount:default:velum
```

---

## 📋 Checklist перед релизом

### Phase 1-2 (MVP)
- [ ] Health check работает
- [ ] Namespaces list < 500ms
- [ ] Events по namespace / involved object для подов и деплойментов
- [ ] Pods list с фильтрами
- [ ] Pod logs streaming
- [ ] Exec terminal стабилен
- [ ] Deployments scale/restart
- [ ] YAML editor с валидацией
- [ ] Ошибки обрабатываются gracefully
- [ ] Basic auth integration

### Phase 3-5 (Core Features)
- [ ] Services CRUD
- [ ] ConfigMaps/Secrets CRUD
- [ ] Ingress visualization
- [ ] PV/PVC binding
- [ ] Jobs/CronJobs
- [ ] RBAC matrix
- [ ] SelfSubjectRulesReview integration

### Phase 6-8 (Advanced)
- [ ] CRD support
- [ ] HPA/VPA
- [ ] Events stream
- [ ] Metrics charts
- [ ] Topology map

### Phase 9-10 (Enterprise)
- [ ] Helm integration
- [ ] Multi-cluster
- [ ] Audit logging
- [ ] Backup/Restore
- [ ] Dark/Light theme
- [ ] Mobile responsive
- [ ] i18n
- [ ] Accessibility

### Security Checklist
- [ ] Kubeconfig шифрование
- [ ] RBAC проверка на каждом endpoint
- [ ] Audit logging деструктивных операций
- [ ] Rate limiting на WebSocket
- [ ] Timeout на exec сессии
- [ ] Secrets masking в UI
- [ ] No credentials в логах

### Performance Checklist
- [ ] API response < 100ms (p50)
- [ ] API response < 500ms (p95)
- [ ] Page load < 1s
- [ ] WebSocket latency < 100ms
- [ ] Memory usage < 500MB
- [ ] CPU usage < 10% в простое

---

## 🎯 Success Metrics

### Adoption Metrics
- **Weekly Active Users** — цель: 100+ через 3 месяца
- **Clusters Connected** — цель: 50+ через 6 месяцев
- **Daily Operations** — цель: 1000+ операций в день

### Technical Metrics
- **API Availability** — цель: 99.9%
- **Error Rate** — цель: < 0.1%
- **WebSocket Reconnect Rate** — цель: < 1% в час
- **Page Load Time** — цель: < 1s (p95)

### User Satisfaction
- **NPS Score** — цель: > 50
- **Support Tickets** — цель: < 5 в неделю
- **Feature Requests** — цель: 10+ в месяц (активное использование)

---

## 🔄 Continuous Improvement

### Feedback Loop
1. **User Feedback** — форма в UI, GitHub Issues
2. **Usage Analytics** — anonymized metrics (opt-in)
3. **Performance Monitoring** — Prometheus + Grafana
4. **Error Tracking** — Sentry или аналог

### Release Cycle
- **MVP (Phase 1-2)** — через 5 недель
- **Beta (Phase 1-5)** — через 9 недель
- **RC (Phase 1-8)** — через 14 недель
- **GA (Phase 1-10)** — через 18 недель

### Post-GA Roadmap
- **v1.1** — Plugins marketplace (как Headlamp)
- **v1.2** — Расширение GitOps (Argo CD/Flux) поверх черновой интеграции [фазы 10](#фазы-реализации)
- **v1.3+** — Multi-cluster advanced, опциональный AI-assistant (условия — в **v1.0.0** раздела Changelog ниже)
- **v2.0** — Крупные пересмотры UX/масштаба при необходимости

---

## 📞 Команда и коммуникация

### Роли
- **Tech Lead** — архитектура, code review
- **Backend Developer** — Rust API handlers
- **Frontend Developer** — Vanilla JS компоненты
- **QA Engineer** — тесты, E2E
- **DevOps** — CI/CD, infrastructure
- **UX Designer** — interface design, accessibility

### Коммуникация
- **Daily Standup** — 15 минут
- **Sprint Planning** — каждые 2 недели
- **Demo** — конец каждого спринта
- **Retrospective** — после каждого релиза

### Инструменты
- **GitHub Projects** — task tracking
- **Discord/Slack** — коммуникация
- **Figma** — дизайн моки
- **Notion/Confluence** — документация

---

## 📝 Changelog

### v0.1.0 (Планируется)
- Namespaces CRUD
- Pods (view, logs, delete)
- Deployments (view, scale, restart)
- Cluster overview

### v0.2.0
- Services, ConfigMaps, Secrets
- YAML editor
- WebSocket events

### v0.3.0
- StatefulSets, DaemonSets
- Jobs, CronJobs
- Ingress, NetworkPolicy

### v0.4.0
- RBAC полный
- Storage (PV, PVC, StorageClass)
- Metrics integration

### v0.5.0
- Helm integration
- Multi-cluster
- Topology visualization

### v1.0.0
- Широкое покрытие core и advanced ресурсов (фиксировать чеклист в release notes; **полный паритет с kubectl** — долгосрочный ориентир из KPI)
- Enterprise: multi-cluster, аудит, безопасный apply / RBAC-UX
- **Опционально:** AI-assistant для troubleshooting — только после отдельного решения по границам доверия и данным

---

## 🔗 Референсы

**UI и платформы**
- [Headlamp](https://headlamp.dev/) — официальный Kubernetes SIG UI; [архитектура](https://headlamp.dev/docs/latest/development/architecture)
- [Kubernetes Dashboard](https://github.com/kubernetes/dashboard) — минимальный in-cluster UI
- [Lens](https://k8slens.dev/) — desktop-клиент, контексты и расширения
- [Rancher](https://www.rancher.com/) — мульти-кластер и проекты
- [Octant](https://octant.dev/) — (архив идей) визуализация ресурсов
- [K9s](https://k9scli.io/) — терминальный UX (клавиатура, фильтры)
- [Argo CD](https://argo-cd.readthedocs.io/) — GitOps UI, diff/sync

**API и клиенты**
- [kube-rs](https://kube.rs/) — Rust-клиент для Kubernetes
- [Kubernetes API Reference](https://kubernetes.io/docs/reference/generated/kubernetes-api/)
- [Server-Side Apply](https://kubernetes.io/docs/reference/using-api/server-side-apply/)

---

## 🚀 Следующие шаги

1. ⏳ Зафиксировать **baseline** в коде и документе (раздел «Текущее состояние репозитория») при каждом крупном изменении модуля `rust/src/kubernetes/`.
2. ⏳ **PoC:** `kube::Client` + list namespaces и проверка прав доступа с той же моделью creds, что будет у UI.
3. ⏳ Выбрать **префикс API** (`/api/kubernetes/...`) и стратегию монтирования роутов в Axum; не конфликтовать с существующими эндпоинтами Velum.
4. ⏳ Реализовать **Фазу 1** (фундамент) из плана выше.
5. ⏳ **Фаза 2** — Pods + Deployments; далее по дорожной карте фаз.

---

*Последнее обновление: 29 марта 2026 — согласование целей и фаз, REST-конвенции, EndpointSlices/Job API, безопасность exec, удаление дубликатов, раздел KPI, стартовая команда `cargo run -- server`.*  
*Статус: В разработке · Следующий review: 5 апреля 2026*
