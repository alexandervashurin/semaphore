//! Бенчмарки для database операций

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use serde::{Deserialize, Serialize};
use std::hint::black_box;
use tokio::runtime::Runtime;

/// Бенчмарк простых операций
fn bench_simple_operations(c: &mut Criterion) {
    // Бенчмарк для базовых операций
    c.bench_function("string_concat", |b| {
        b.iter(|| {
            let s1 = "hello";
            let s2 = "world";
            black_box(format!("{}:{}", s1, s2))
        })
    });

    c.bench_function("vec_push_1000", |b| {
        b.iter(|| {
            let mut vec: Vec<i32> = Vec::with_capacity(1000);
            for i in 0..1000 {
                vec.push(i);
            }
            black_box(vec.len())
        })
    });
}

/// Бенчмарк сериализации JSON
fn bench_json_serialization(c: &mut Criterion) {
    #[derive(Serialize, Deserialize, Clone)]
    struct TestData {
        id: i64,
        name: String,
        value: f64,
    }

    let data = TestData {
        id: 123,
        name: "test".to_string(),
        value: 42.0,
    };

    c.bench_function("json_serialize", |b| {
        b.iter(|| black_box(serde_json::to_string(&data).unwrap()))
    });

    let json = r#"{"id":123,"name":"test","value":42.0}"#;

    c.bench_function("json_deserialize", |b| {
        b.iter(|| {
            let _: TestData = black_box(serde_json::from_str(json).unwrap());
        })
    });
}

/// Бенчмарк хэширования
fn bench_hashing(c: &mut Criterion) {
    use sha2::{Digest, Sha256};

    let data = b"test data for hashing";

    c.bench_function("sha256_hash", |b| {
        b.iter(|| {
            let mut hasher = Sha256::new();
            hasher.update(black_box(data));
            black_box(hasher.finalize())
        })
    });
}

/// Бенчмарк работы с HashMap
fn bench_hashmap(c: &mut Criterion) {
    use std::collections::HashMap;

    c.bench_function("hashmap_insert_100", |b| {
        b.iter(|| {
            let mut map: HashMap<String, i32> = HashMap::new();
            for i in 0..100 {
                map.insert(format!("key_{}", i), i);
            }
            black_box(map.len())
        })
    });

    c.bench_function("hashmap_get_100", |b| {
        let mut map: HashMap<String, i32> = HashMap::new();
        for i in 0..100 {
            map.insert(format!("key_{}", i), i);
        }

        b.iter(|| {
            for i in 0..100 {
                black_box(map.get(&format!("key_{}", i)));
            }
        })
    });
}

/// Бенчмарк Arc<RwLock> паттерна
fn bench_arc_rwlock(c: &mut Criterion) {
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let rt = Runtime::new().unwrap();
    let data = Arc::new(RwLock::new(vec![1, 2, 3, 4, 5]));

    c.bench_function("arc_rwlock_read", |b| {
        b.iter(|| {
            rt.block_on(async {
                let guard = data.read().await;
                black_box(guard.len())
            })
        })
    });
}

criterion_group!(
    benches,
    bench_simple_operations,
    bench_json_serialization,
    bench_hashing,
    bench_hashmap,
    bench_arc_rwlock,
);

criterion_main!(benches);
