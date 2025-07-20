use ai_manager_shared::errors::SystemError;
use async_trait::async_trait;
use sqlx::{Column, Pool, Postgres, Row, Sqlite};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
}

#[async_trait]
pub trait DatabaseConnection: Send + Sync {
    async fn execute(&self, query: &str) -> Result<(), SystemError>;
    async fn execute_with_params(
        &self,
        query: &str,
        params: Vec<&(dyn sqlx::Encode<sqlx::Any> + Send + Sync)>,
    ) -> Result<(), SystemError>;
    async fn fetch_one_json(&self, query: &str) -> Result<Option<serde_json::Value>, SystemError>;
    async fn fetch_all_json(&self, query: &str) -> Result<Vec<serde_json::Value>, SystemError>;
    async fn health_check(&self) -> Result<(), SystemError>;
}

pub struct SqliteConnection {
    pool: Pool<Sqlite>,
}

impl SqliteConnection {
    pub async fn new(database_url: &str) -> Result<Self, SystemError> {
        let pool = sqlx::SqlitePool::connect(database_url)
            .await
            .map_err(|e| SystemError::Database(format!("Failed to connect to SQLite: {}", e)))?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseConnection for SqliteConnection {
    async fn execute(&self, query: &str) -> Result<(), SystemError> {
        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("SQLite execute error: {}", e)))?;
        Ok(())
    }

    async fn execute_with_params(
        &self,
        query: &str,
        _params: Vec<&(dyn sqlx::Encode<sqlx::Any> + Send + Sync)>,
    ) -> Result<(), SystemError> {
        // For simplicity, we'll implement this without params for now
        self.execute(query).await
    }

    async fn fetch_one_json(&self, query: &str) -> Result<Option<serde_json::Value>, SystemError> {
        let row = sqlx::query(query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("SQLite fetch error: {}", e)))?;

        if let Some(row) = row {
            let mut json = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = match row.try_get::<String, _>(i) {
                    Ok(s) => serde_json::Value::String(s),
                    Err(_) => {
                        // Try as integer
                        match row.try_get::<i64, _>(i) {
                            Ok(n) => serde_json::Value::Number(serde_json::Number::from(n)),
                            Err(_) => {
                                // Try as float
                                match row.try_get::<f64, _>(i) {
                                    Ok(f) => serde_json::Number::from_f64(f)
                                        .map(serde_json::Value::Number)
                                        .unwrap_or(serde_json::Value::Null),
                                    Err(_) => serde_json::Value::Null,
                                }
                            }
                        }
                    }
                };
                json.insert(column.name().to_string(), value);
            }
            Ok(Some(serde_json::Value::Object(json)))
        } else {
            Ok(None)
        }
    }

    async fn fetch_all_json(&self, query: &str) -> Result<Vec<serde_json::Value>, SystemError> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("SQLite fetch error: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            let mut json = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = match row.try_get::<String, _>(i) {
                    Ok(s) => serde_json::Value::String(s),
                    Err(_) => {
                        // Try as integer
                        match row.try_get::<i64, _>(i) {
                            Ok(n) => serde_json::Value::Number(serde_json::Number::from(n)),
                            Err(_) => {
                                // Try as float
                                match row.try_get::<f64, _>(i) {
                                    Ok(f) => serde_json::Number::from_f64(f)
                                        .map(serde_json::Value::Number)
                                        .unwrap_or(serde_json::Value::Null),
                                    Err(_) => serde_json::Value::Null,
                                }
                            }
                        }
                    }
                };
                json.insert(column.name().to_string(), value);
            }
            results.push(serde_json::Value::Object(json));
        }
        Ok(results)
    }

    async fn health_check(&self) -> Result<(), SystemError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("SQLite health check failed: {}", e)))?;
        Ok(())
    }
}

pub struct PostgresConnection {
    pool: Pool<Postgres>,
}

impl PostgresConnection {
    pub async fn new(database_url: &str) -> Result<Self, SystemError> {
        let pool = sqlx::PgPool::connect(database_url).await.map_err(|e| {
            SystemError::Database(format!("Failed to connect to PostgreSQL: {}", e))
        })?;

        Ok(Self { pool })
    }
}

#[async_trait]
impl DatabaseConnection for PostgresConnection {
    async fn execute(&self, query: &str) -> Result<(), SystemError> {
        sqlx::query(query)
            .execute(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("PostgreSQL execute error: {}", e)))?;
        Ok(())
    }

    async fn execute_with_params(
        &self,
        query: &str,
        _params: Vec<&(dyn sqlx::Encode<sqlx::Any> + Send + Sync)>,
    ) -> Result<(), SystemError> {
        // For simplicity, we'll implement this without params for now
        self.execute(query).await
    }

    async fn fetch_one_json(&self, query: &str) -> Result<Option<serde_json::Value>, SystemError> {
        let row = sqlx::query(query)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("PostgreSQL fetch error: {}", e)))?;

        if let Some(row) = row {
            let mut json = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = match row.try_get::<String, _>(i) {
                    Ok(s) => serde_json::Value::String(s),
                    Err(_) => {
                        // Try as integer
                        match row.try_get::<i64, _>(i) {
                            Ok(n) => serde_json::Value::Number(serde_json::Number::from(n)),
                            Err(_) => {
                                // Try as float
                                match row.try_get::<f64, _>(i) {
                                    Ok(f) => serde_json::Number::from_f64(f)
                                        .map(serde_json::Value::Number)
                                        .unwrap_or(serde_json::Value::Null),
                                    Err(_) => serde_json::Value::Null,
                                }
                            }
                        }
                    }
                };
                json.insert(column.name().to_string(), value);
            }
            Ok(Some(serde_json::Value::Object(json)))
        } else {
            Ok(None)
        }
    }

    async fn fetch_all_json(&self, query: &str) -> Result<Vec<serde_json::Value>, SystemError> {
        let rows = sqlx::query(query)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("PostgreSQL fetch error: {}", e)))?;

        let mut results = Vec::new();
        for row in rows {
            let mut json = serde_json::Map::new();
            for (i, column) in row.columns().iter().enumerate() {
                let value = match row.try_get::<String, _>(i) {
                    Ok(s) => serde_json::Value::String(s),
                    Err(_) => {
                        // Try as integer
                        match row.try_get::<i64, _>(i) {
                            Ok(n) => serde_json::Value::Number(serde_json::Number::from(n)),
                            Err(_) => {
                                // Try as float
                                match row.try_get::<f64, _>(i) {
                                    Ok(f) => serde_json::Number::from_f64(f)
                                        .map(serde_json::Value::Number)
                                        .unwrap_or(serde_json::Value::Null),
                                    Err(_) => serde_json::Value::Null,
                                }
                            }
                        }
                    }
                };
                json.insert(column.name().to_string(), value);
            }
            results.push(serde_json::Value::Object(json));
        }
        Ok(results)
    }

    async fn health_check(&self) -> Result<(), SystemError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| SystemError::Database(format!("PostgreSQL health check failed: {}", e)))?;
        Ok(())
    }
}

pub async fn create_connection(
    db_type: DatabaseType,
    database_url: &str,
) -> Result<Arc<dyn DatabaseConnection>, SystemError> {
    match db_type {
        DatabaseType::SQLite => {
            let conn = SqliteConnection::new(database_url).await?;
            Ok(Arc::new(conn))
        }
        DatabaseType::PostgreSQL => {
            let conn = PostgresConnection::new(database_url).await?;
            Ok(Arc::new(conn))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sqlite_connection() {
        let conn = create_connection(DatabaseType::SQLite, ":memory:").await;
        assert!(conn.is_ok());

        if let Ok(conn) = conn {
            let health = conn.health_check().await;
            assert!(health.is_ok());
        }
    }

    #[tokio::test]
    async fn test_sqlite_basic_operations() {
        let conn = create_connection(DatabaseType::SQLite, ":memory:")
            .await
            .expect("Failed to create connection");

        // Create a test table
        conn.execute("CREATE TABLE test (id INTEGER PRIMARY KEY, name TEXT)")
            .await
            .expect("Failed to create table");

        // Insert test data
        conn.execute("INSERT INTO test (name) VALUES ('test_name')")
            .await
            .expect("Failed to insert data");

        // Query data
        let result = conn
            .fetch_all_json("SELECT * FROM test")
            .await
            .expect("Failed to query data");

        assert!(!result.is_empty());
    }
}
