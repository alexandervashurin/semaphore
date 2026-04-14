//! Runners Module
//!
//! Модуль раннеров для Velum

pub mod job_pool;
pub mod running_job;
pub mod task_queue;
pub mod types;

pub use job_pool::{Job, JobLogger, JobPool};
pub use running_job::RunningJob;
pub use task_queue::{build_task_queue, InMemoryTaskQueue, RedisTaskQueue, TaskQueue};
pub use types::{
    CommitInfo, JobData, JobProgress, JobState, LogRecord, RunnerProgress, RunnerRegistration,
    RunnerState,
};
