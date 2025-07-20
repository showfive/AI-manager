use crate::connection::DatabaseConnection;
use ai_manager_shared::errors::SystemError;

const MIGRATIONS: &[&str] = &[
    // Migration 001: Create conversations table
    r#"
    CREATE TABLE IF NOT EXISTS conversations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        user_id TEXT NOT NULL,
        messages TEXT NOT NULL,
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    "#,
    // Migration 002: Create user_profiles table
    r#"
    CREATE TABLE IF NOT EXISTS user_profiles (
        id TEXT PRIMARY KEY,
        name TEXT,
        email TEXT,
        preferences TEXT NOT NULL DEFAULT '{}',
        created_at TEXT NOT NULL,
        updated_at TEXT NOT NULL
    );
    "#,
    // Migration 003: Create indexes for better performance
    r#"
    CREATE INDEX IF NOT EXISTS idx_conversations_user_id ON conversations(user_id);
    "#,
    r#"
    CREATE INDEX IF NOT EXISTS idx_conversations_created_at ON conversations(created_at);
    "#,
    r#"
    CREATE INDEX IF NOT EXISTS idx_user_profiles_email ON user_profiles(email);
    "#,
];

pub async fn run_migrations(connection: &dyn DatabaseConnection) -> Result<(), SystemError> {
    // Create migrations table to track applied migrations
    connection
        .execute(
            r#"
        CREATE TABLE IF NOT EXISTS migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            migration_name TEXT NOT NULL,
            applied_at TEXT NOT NULL
        );
        "#,
        )
        .await?;

    // Get list of applied migrations
    let applied_migrations = match connection
        .fetch_all_json("SELECT migration_name FROM migrations")
        .await
    {
        Ok(rows) => rows
            .into_iter()
            .filter_map(|row| {
                row.get("migration_name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect::<Vec<_>>(),
        Err(_) => Vec::new(),
    };

    // Apply migrations that haven't been applied yet
    for (index, migration_sql) in MIGRATIONS.iter().enumerate() {
        let migration_name = format!("migration_{:03}", index + 1);

        if !applied_migrations.contains(&migration_name) {
            connection.execute(migration_sql).await?;

            // Record migration as applied
            let insert_sql = format!(
                "INSERT INTO migrations (migration_name, applied_at) VALUES ('{}', '{}')",
                migration_name,
                chrono::Utc::now().to_rfc3339()
            );
            connection.execute(&insert_sql).await?;

            tracing::info!("Applied migration: {}", migration_name);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connection::{create_connection, DatabaseType};

    #[tokio::test]
    async fn test_migrations() {
        let connection = create_connection(DatabaseType::SQLite, ":memory:")
            .await
            .expect("Failed to create connection");

        // Run migrations
        let result = run_migrations(&*connection).await;
        assert!(result.is_ok());

        // Verify tables were created
        let tables = connection
            .fetch_all_json("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .await
            .expect("Failed to query tables");

        let table_names: Vec<String> = tables
            .into_iter()
            .filter_map(|row| {
                row.get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            })
            .collect();

        assert!(table_names.contains(&"conversations".to_string()));
        assert!(table_names.contains(&"user_profiles".to_string()));
        assert!(table_names.contains(&"migrations".to_string()));
    }

    #[tokio::test]
    async fn test_migration_idempotency() {
        let connection = create_connection(DatabaseType::SQLite, ":memory:")
            .await
            .expect("Failed to create connection");

        // Run migrations twice
        let result1 = run_migrations(&*connection).await;
        assert!(result1.is_ok());

        let result2 = run_migrations(&*connection).await;
        assert!(result2.is_ok());

        // Verify migrations table has correct number of entries
        let migration_count = connection
            .fetch_all_json("SELECT COUNT(*) as count FROM migrations")
            .await
            .expect("Failed to count migrations");

        // Should have exactly as many migrations as we defined
        assert_eq!(migration_count.len(), 1);
    }
}
