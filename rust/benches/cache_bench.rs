//! Бенчмарки для cache модуля

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;

/// Бенчмарк формирования ключей кэша
fn bench_cache_key(c: &mut Criterion) {
    c.bench_function("cache_key_simple", |b| {
        b.iter(|| cache_key(&["user", "123"]))
    });

    c.bench_function("cache_key_complex", |b| {
        b.iter(|| cache_key(&["api", "v1", "projects", "123", "tasks", "456"]))
    });
}

/// Бенчмарк статистики кэша
fn bench_cache_stats(c: &mut Criterion) {
    c.bench_function("cache_stats_hit_ratio", |b| {
        b.iter(|| {
            let mut stats = CacheStats::default();
            stats.hits = 80;
            stats.misses = 20;
            black_box(stats.hit_ratio())
        })
    });

    c.bench_function("cache_stats_total_requests", |b| {
        b.iter(|| {
            let mut stats = CacheStats::default();
            stats.hits = 80;
            stats.misses = 20;
            black_box(stats.total_requests())
        })
    });
}

/// Бенчмарк создания Redis конфигурации
fn bench_redis_config(c: &mut Criterion) {
    c.bench_function("redis_config_default", |b| {
        b.iter(|| RedisConfig::default())
    });

    c.bench_function("redis_config_custom", |b| {
        b.iter(|| RedisConfig {
            url: "redis://localhost:6379".to_string(),
            key_prefix: "test:".to_string(),
            default_ttl_secs: 300,
            max_retries: 3,
            connection_timeout_secs: 5,
            enabled: true,
        })
    });
}

/// Бенчмарк создания Redis кэша
fn bench_redis_cache_creation(c: &mut Criterion) {
    c.bench_function("redis_cache_new", |b| {
        b.iter(|| {
            let config = RedisConfig::default();
            black_box(RedisCache::new(config))
        })
    });
}

/// Бенчмарк is_enabled проверки
fn bench_cache_is_enabled(c: &mut Criterion) {
    let config_disabled = RedisConfig::default();
    let cache_disabled = RedisCache::new(config_disabled);

    let config_enabled = RedisConfig {
        enabled: true,
        ..Default::default()
    };
    let cache_enabled = RedisCache::new(config_enabled);

    c.bench_function("cache_is_enabled_false", |b| {
        b.iter(|| black_box(cache_disabled.is_enabled()))
    });

    c.bench_function("cache_is_enabled_true", |b| {
        b.iter(|| black_box(cache_enabled.is_enabled()))
    });
}

criterion_group!(
    benches,
    bench_cache_key,
    bench_cache_stats,
    bench_redis_config,
    bench_redis_cache_creation,
    bench_cache_is_enabled,
);

criterion_main!(benches);

use velum_ffi::cache::{cache_key, CacheStats, RedisCache, RedisConfig};
