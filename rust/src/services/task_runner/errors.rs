//! TaskRunner Errors - обработка ошибок
//!
//! Аналог services/tasks/task_runner_error.go из Go версии

use crate::error::Result;
use crate::services::task_runner::TaskRunner;

impl TaskRunner {
    /// prepare_error подготавливает ошибку для логирования
    pub fn prepare_error(&self, err: Result<()>, error_msg: &str) -> Result<()> {
        if let Err(e) = err {
            // Логирование ошибки
            self.log(&format!("{}: {}", error_msg, e));

            // Добавление контекста
            return Err(crate::error::Error::Other(format!("{}: {}", error_msg, e)));
        }

        Ok(())
    }

    /// is_error_fatal проверяет, является ли ошибка фатальной
    pub fn is_error_fatal(&self, err: &crate::error::Error) -> bool {
        let err_str = err.to_string().to_lowercase();

        // Список фатальных ошибок
        let fatal_errors = [
            "permission denied",
            "authentication failed",
            "connection refused",
            "no such file",
            "command not found",
        ];

        fatal_errors.iter().any(|fatal| err_str.contains(fatal))
    }

    /// log_error логирует ошибку с контекстом
    pub fn log_error(&self, err: &crate::error::Error, context: &str) {
        let msg = format!("{}: {}", context, err);
        self.log(&msg);

        use tracing::error;
        error!(
            task_id = self.task.id,
            context = context,
            "Task error: {}",
            err
        );
    }

    /// handle_error обрабатывает ошибку задачи
    pub async fn handle_error(&mut self, err: crate::error::Error) {
        self.log_error(&err, "Task execution failed");

        // Проверка на фатальную ошибку
        if self.is_error_fatal(&err) {
            self.log("Fatal error detected");
            self.set_status(crate::services::task_logger::TaskStatus::Error)
                .await;
        } else {
            self.log("Non-fatal error, continuing...");
        }
    }

    /// wrap_error оборачивает ошибку с дополнительным сообщением
    pub fn wrap_error(&self, err: crate::error::Error, message: &str) -> crate::error::Error {
        crate::error::Error::Other(format!("{}: {}", message, err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::MockStore;
    use crate::db_lib::AccessKeyInstallerImpl;
    use crate::models::Task;
    use crate::services::task_logger::TaskStatus;
    use crate::services::task_pool::TaskPool;
    use chrono::Utc;
    use std::sync::Arc;

    fn create_test_task_runner() -> TaskRunner {
        let mut task = Task::default();
        task.id = 1;
        task.template_id = 1;
        task.project_id = 1;
        task.created = Utc::now();

        let pool = Arc::new(TaskPool::new(Arc::new(MockStore::new()), 5));

        TaskRunner::new(
            task,
            pool,
            "testuser".to_string(),
            AccessKeyInstallerImpl::new(),
        )
    }

    #[tokio::test]
    async fn test_is_error_fatal_permission_denied() {
        let runner = create_test_task_runner();
        let err = crate::error::Error::Other("permission denied".to_string());
        assert!(runner.is_error_fatal(&err));
    }

    #[tokio::test]
    async fn test_is_error_fatal_non_fatal() {
        let runner = create_test_task_runner();
        let err = crate::error::Error::Other("minor issue".to_string());
        assert!(!runner.is_error_fatal(&err));
    }

    #[tokio::test]
    async fn test_wrap_error() {
        let runner = create_test_task_runner();
        let err = crate::error::Error::Other("original error".to_string());
        let wrapped = runner.wrap_error(err, "context");
        assert!(wrapped.to_string().contains("context"));
        assert!(wrapped.to_string().contains("original error"));
    }

    #[tokio::test]
    async fn test_prepare_error_ok() {
        let runner = create_test_task_runner();
        let result = runner.prepare_error(Ok(()), "test message");
        assert!(result.is_ok());
    }
}
