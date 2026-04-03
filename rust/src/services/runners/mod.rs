//! Runners Module
//!
//! Модуль раннеров для Velum

pub mod job_pool;
pub mod running_job;
pub mod task_queue;
pub mod types;

pub use job_pool::{JobPool, Job, JobLogger};
pub use running_job::RunningJob;
pub use task_queue::{TaskQueue, InMemoryTaskQueue, RedisTaskQueue, build_task_queue};
pub use types::{
    JobData, RunnerState, RunnerProgress, JobProgress,
    JobState, LogRecord, CommitInfo, RunnerRegistration,
};
