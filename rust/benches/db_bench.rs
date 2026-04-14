//! Бенчмарки для database операций Velum
//!
//! Замеряет производительность CRUD-операций через MockStore
//! (HashMap + RwLock backend).
//!
//! Запуск: `cargo bench --bench db_bench`

use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use tokio::runtime::Runtime;

use velum_ffi::db::MockStore;
use velum_ffi::db::store::*;
use velum_ffi::models::*;

/// Бенчмарк CRUD операций с шаблонами
fn bench_template_crud(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // CREATE: создание 100 шаблонов
    c.bench_function("template/create_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let mut tpl = Template::default();
                    tpl.id = i;
                    tpl.project_id = 1;
                    tpl.name = format!("template_{}", i);
                    let _: Template = black_box(store.create_template(tpl).await.unwrap());
                }
            });
        });
    });

    // READ: получение 100 шаблонов по одному
    c.bench_function("template/get_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let mut tpl = Template::default();
                    tpl.id = i;
                    tpl.project_id = 1;
                    tpl.name = format!("tpl_{}", i);
                    let _: Template = store.create_template(tpl).await.unwrap();
                }
                for i in 0..100i32 {
                    let _: Template = black_box(store.get_template(1, i).await.unwrap());
                }
            });
        });
    });

    // LIST: получение списка 100 шаблонов
    c.bench_function("template/list_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let mut tpl = Template::default();
                    tpl.id = i;
                    tpl.project_id = 1;
                    tpl.name = format!("tpl_{}", i);
                    let _: Template = store.create_template(tpl).await.unwrap();
                }
                let _: Vec<Template> = black_box(store.get_templates(1).await.unwrap());
            });
        });
    });

    // UPDATE: обновление 100 шаблонов
    c.bench_function("template/update_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let mut tpl = Template::default();
                    tpl.id = i;
                    tpl.project_id = 1;
                    tpl.name = format!("tpl_{}", i);
                    store.create_template(tpl).await.unwrap();
                }
                for i in 0..100i32 {
                    let mut tpl: Template = store.get_template(1, i).await.unwrap();
                    tpl.name = format!("updated_{}", i);
                    black_box(store.update_template(tpl).await.unwrap());
                }
            });
        });
    });
}

/// Бенчмарк CRUD операций с задачами
fn bench_task_crud(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    use velum_ffi::services::task_logger::TaskStatus;

    c.bench_function("task/create_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let task = Task {
                        id: i,
                        project_id: 1,
                        template_id: 1,
                        created: chrono::Utc::now(),
                        status: TaskStatus::Waiting,
                        playbook: None,
                        environment: None,
                        secret: None,
                        arguments: None,
                        git_branch: None,
                        user_id: None,
                        integration_id: None,
                        schedule_id: None,
                        start: None,
                        end: None,
                        message: None,
                        commit_hash: None,
                        commit_message: None,
                        build_task_id: None,
                        version: None,
                        inventory_id: None,
                        repository_id: None,
                        environment_id: None,
                        params: None,
                    };
                    let _: Task = store.create_task(task).await.unwrap();
                }
            });
        });
    });

    c.bench_function("task/get_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let task = Task {
                        id: i,
                        project_id: 1,
                        template_id: 1,
                        created: chrono::Utc::now(),
                        status: TaskStatus::Waiting,
                        playbook: None,
                        environment: None,
                        secret: None,
                        arguments: None,
                        git_branch: None,
                        user_id: None,
                        integration_id: None,
                        schedule_id: None,
                        start: None,
                        end: None,
                        message: None,
                        commit_hash: None,
                        commit_message: None,
                        build_task_id: None,
                        version: None,
                        inventory_id: None,
                        repository_id: None,
                        environment_id: None,
                        params: None,
                    };
                    let _: Task = store.create_task(task).await.unwrap();
                }
                for i in 0..100i32 {
                    let _: Task = black_box(store.get_task(1, i).await.unwrap());
                }
            });
        });
    });
}

/// Бенчмарк CRUD операций с инвентарями
fn bench_inventory_crud(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("inventory/create_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let inv = Inventory {
                        id: i,
                        project_id: 1,
                        name: format!("inv_{}", i),
                        ..Default::default()
                    };
                    let _: Inventory = black_box(store.create_inventory(inv).await.unwrap());
                }
            });
        });
    });

    c.bench_function("inventory/get_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let inv = Inventory {
                        id: i,
                        project_id: 1,
                        name: format!("inv_{}", i),
                        ..Default::default()
                    };
                    let _: Inventory = store.create_inventory(inv).await.unwrap();
                }
                for i in 0..100i32 {
                    let _: Inventory = black_box(store.get_inventory(1, i).await.unwrap());
                }
            });
        });
    });
}

/// Бенчмарк CRUD операций с окружениями
fn bench_environment_crud(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("environment/create_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let env = Environment {
                        id: i,
                        project_id: 1,
                        name: format!("env_{}", i),
                        ..Default::default()
                    };
                    let _: Environment = black_box(store.create_environment(env).await.unwrap());
                }
            });
        });
    });

    c.bench_function("environment/list_100", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..100i32 {
                    let env = Environment {
                        id: i,
                        project_id: 1,
                        name: format!("env_{}", i),
                        ..Default::default()
                    };
                    let _: Environment = store.create_environment(env).await.unwrap();
                }
                let _: Vec<Environment> = black_box(store.get_environments(1).await.unwrap());
            });
        });
    });
}

/// Бенчмарк событий (events)
fn bench_event_crud(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("event/create_1000", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..1000i32 {
                    let event = Event {
                        id: i,
                        object_type: "task".to_string(),
                        object_id: Some(i),
                        project_id: Some(1),
                        description: format!("Task {} completed", i),
                        user_id: Some(1),
                        created: chrono::Utc::now(),
                    };
                    let _: Event = black_box(store.create_event(event).await.unwrap());
                }
            });
        });
    });

    c.bench_function("event/list_100_from_1000", |b| {
        b.iter(|| {
            let store = MockStore::new();
            rt.block_on(async {
                for i in 0..1000i32 {
                    let event = Event {
                        id: i,
                        object_type: "task".to_string(),
                        object_id: Some(i),
                        project_id: Some(1),
                        description: format!("Task {} completed", i),
                        user_id: Some(1),
                        created: chrono::Utc::now(),
                    };
                    let _: Event = store.create_event(event).await.unwrap();
                }
                let _: Vec<Event> = black_box(store.get_events(Some(1), 100).await.unwrap());
            });
        });
    });
}

criterion_group!(
    benches,
    bench_template_crud,
    bench_task_crud,
    bench_inventory_crud,
    bench_environment_crud,
    bench_event_crud,
);

criterion_main!(benches);
