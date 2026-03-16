//! Вспомогательные функции для тестов SQL-слоя

#[cfg(test)]
pub async fn create_test_pool() -> Result<
    (sqlx::SqlitePool, tempfile::NamedTempFile),
    crate::error::Error,
> {
    use sqlx::sqlite::SqlitePoolOptions;

    let (url, temp) = super::init::test_sqlite_url();
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .map_err(crate::error::Error::Database)?;
    Ok((pool, temp))
}

#[cfg(test)]
pub async fn init_user_table(pool: &sqlx::SqlitePool) -> Result<(), crate::error::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS user (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL,
            name TEXT NOT NULL,
            email TEXT NOT NULL,
            password TEXT NOT NULL,
            admin INTEGER NOT NULL,
            external INTEGER NOT NULL,
            alert INTEGER NOT NULL,
            pro INTEGER NOT NULL,
            created DATETIME NOT NULL
        )",
    )
    .execute(pool)
    .await
    .map_err(crate::error::Error::Database)?;
    Ok(())
}

#[cfg(test)]
pub async fn init_project_table(pool: &sqlx::SqlitePool) -> Result<(), crate::error::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS project (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            created DATETIME NOT NULL,
            alert INTEGER NOT NULL,
            alert_chat TEXT,
            max_parallel_tasks INTEGER NOT NULL,
            type TEXT NOT NULL,
            default_secret_storage_id INTEGER
        )",
    )
    .execute(pool)
    .await
    .map_err(crate::error::Error::Database)?;
    Ok(())
}

#[cfg(test)]
pub async fn init_template_table(pool: &sqlx::SqlitePool) -> Result<(), crate::error::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS template (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            playbook TEXT NOT NULL,
            description TEXT NOT NULL,
            inventory_id INTEGER,
            repository_id INTEGER,
            environment_id INTEGER,
            type TEXT NOT NULL,
            app TEXT NOT NULL,
            git_branch TEXT,
            created DATETIME NOT NULL,
            arguments TEXT,
            vault_key_id INTEGER,
            view_id INTEGER,
            build_template_id INTEGER,
            autorun INTEGER NOT NULL DEFAULT 0,
            allow_override_args_in_task INTEGER NOT NULL DEFAULT 0,
            allow_override_branch_in_task INTEGER NOT NULL DEFAULT 0,
            allow_inventory_in_task INTEGER NOT NULL DEFAULT 0,
            allow_parallel_tasks INTEGER NOT NULL DEFAULT 0,
            suppress_success_alerts INTEGER NOT NULL DEFAULT 0,
            task_params TEXT,
            survey_vars TEXT,
            vaults TEXT
        )",
    )
    .execute(pool)
    .await
    .map_err(crate::error::Error::Database)?;
    Ok(())
}
