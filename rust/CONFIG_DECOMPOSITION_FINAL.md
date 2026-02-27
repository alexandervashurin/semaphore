# 📋 Декомпозиция config.go

**Файл**: `util/config.go` (1407 строк)
**Статус**: 🚧 ТРЕБУЕТ ДЕКОМПОЗИЦИИ

---

## 🎯 План Декомпозиции

### 1. Основные компоненты:

#### A. Config Structure (~200 строк)
- ConfigType структура
- DbConfig, LdapConfig, LdapMappings
- OidcProvider, OidcEndpoint
- Constance и переменные

#### B. Config Loading (~300 строк)
- loadConfig()
- loadFromFile()
- loadFromEnv()
- mergeConfigs()

#### C. Config Validation (~150 строк)
- validateConfig()
- validateConfigWithWarnings()
- Validate trait

#### D. Config Helpers (~250 строк)
- GetProjectTmpDir()
- ClearTmpDir()
- GetConfigPath()
- etc.

#### E. LDAP Config (~100 строк)
- LdapConfigFull
- loadLdapFromEnv()

#### F. OIDC Config (~150 строк)
- OidcProvider
- loadOidcFromEnv()

#### G. HA Config (~100 строк)
- HAConfigFull
- HARedisConfigFull
- loadHaFromEnv()

#### H. Logging Config (~50 строк)
- LoggingConfig
- LogFormat, LogLevel
- loadLoggingFromEnv()

#### I. Defaults (~100 строк)
- loadDefaults()
- applyDefaults()
- createDefaultConfig()

---

## 📁 Предлагаемая структура:

```
rust/src/config/
├── mod.rs (главный модуль)
├── types.rs (основные типы)
├── loader.rs (загрузка)
├── validator.rs (валидация)
├── helpers.rs (хелперы)
├── config_ldap.rs (LDAP)
├── config_oidc.rs (OIDC)
├── config_ha.rs (HA)
├── config_logging.rs (Logging)
└── defaults.rs (Defaults)
```

---

## ✅ Уже мигрировано:

- ✅ config_ldap.rs
- ✅ config_oidc.rs
- ✅ config_ha.rs
- ✅ config_logging.rs
- ✅ defaults.rs
- ✅ loader.rs
- ✅ validator.rs
- ✅ types.rs
- ✅ config_helpers.rs
- ✅ config_dirs.rs

---

## ⏳ Осталось:

- ⏳ config.go (основной файл) - можно удалить после проверки
- ⏳ config_auth.go - простая аутентификация
- ⏳ config_sysproc.go - системные процессы
- ⏳ config_sysproc_windows.go - Windows специфичное

---

**Время оценки**: 2-3 часа
**Приоритет**: Высокий (последний шаг к 97%)
