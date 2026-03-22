//! ViewManager - управление представлениями

use crate::db::sql::SqlStore;
use crate::db::store::*;
use crate::error::{Error, Result};
use crate::models::View;
use async_trait::async_trait;

#[async_trait]
impl ViewManager for SqlStore {
    async fn get_views(&self, project_id: i32) -> Result<Vec<View>> { self.db.get_views(project_id).await }
    async fn get_view(&self, project_id: i32, view_id: i32) -> Result<View> { self.db.get_view(project_id, view_id).await }
    async fn create_view(&self, view: View) -> Result<View> { self.db.create_view(view).await }
    async fn update_view(&self, view: View) -> Result<()> { self.db.update_view(view).await }
    async fn delete_view(&self, project_id: i32, view_id: i32) -> Result<()> { self.db.delete_view(project_id, view_id).await }

    async fn set_view_positions(&self, project_id: i32, positions: Vec<(i32, i32)>) -> Result<()> {
        // positions: Vec<(view_id, position)>
        for (view_id, position) in positions {
            let query = "UPDATE view SET position = $1 WHERE id = $2 AND project_id = $3";
                sqlx::query(query)
                    .bind(position)
                    .bind(view_id)
                    .bind(project_id)
                    .execute(self.get_postgres_pool()?)
                    .await
                    .map_err(Error::Database)?;
        }
        Ok(())
    }
}

