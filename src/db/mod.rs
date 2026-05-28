//! Database module: connection pool setup, embedded migrations, and helpers.

pub mod models;
pub mod schema;

use diesel::SqliteConnection;
use diesel_async::AsyncConnection as _;
use diesel_async::SimpleAsyncConnection as _;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// The async SQLite connection type used throughout the application.
pub type AsyncSqliteConnection = SyncConnectionWrapper<SqliteConnection>;

/// The bb8 connection pool type.
pub type DbPool = Pool<AsyncSqliteConnection>;

/// Build a bb8 connection pool for the given SQLite database path.
///
/// Also runs any pending migrations (via a one-shot sync connection) before
/// returning the async pool.
///
/// Each connection is initialised with:
/// - `journal_mode=WAL`     — readers and the writer no longer block each other.
/// - `busy_timeout=5000`    — wait up to 5 s on lock contention.
/// - `synchronous=NORMAL`   — safe with WAL; faster than the default FULL.
/// - `foreign_keys=ON`      — SQLite does not enforce FK constraints by default.
pub async fn build_pool(database_url: &str) -> DbPool {
    // Run migrations synchronously before handing control to the async pool.
    {
        use diesel::Connection;
        let mut conn = SqliteConnection::establish(database_url)
            .expect("Failed to open database for migrations");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run database migrations");
    }

    let db_url = database_url.to_owned();
    let mut manager_config =
        diesel_async::pooled_connection::ManagerConfig::<AsyncSqliteConnection>::default();
    manager_config.custom_setup = Box::new(move |url: &str| {
        let url = url.to_owned();
        Box::pin(async move {
            let mut conn = AsyncSqliteConnection::establish(&url).await?;
            conn.batch_execute(
                "PRAGMA journal_mode=WAL;
                 PRAGMA busy_timeout=5000;
                 PRAGMA synchronous=NORMAL;
                 PRAGMA foreign_keys=ON;",
            )
            .await
            .map_err(diesel::result::ConnectionError::CouldntSetupConfiguration)?;
            Ok(conn)
        })
    });

    let manager = AsyncDieselConnectionManager::new_with_config(&db_url, manager_config);
    Pool::builder()
        .build(manager)
        .await
        .expect("Failed to create async database connection pool")
}
