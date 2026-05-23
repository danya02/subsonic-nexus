//! Database module: connection pool setup, embedded migrations, and helpers.

pub mod models;
pub mod schema;

use diesel::SqliteConnection;
use diesel::connection::SimpleConnection;
use diesel::r2d2::{self, ConnectionManager, CustomizeConnection, Error as R2d2Error};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;
pub type PooledConn = r2d2::PooledConnection<ConnectionManager<SqliteConnection>>;

/// Applied to every connection as soon as it is checked out of the pool.
///
/// - `journal_mode=WAL`     — readers and the writer no longer block each other;
///                            essential for concurrent dashboard + scan access.
/// - `busy_timeout=5000`    — wait up to 5 s on lock contention before returning
///                            SQLITE_BUSY, instead of failing immediately.
/// - `synchronous=NORMAL`   — safe with WAL; much faster than the default FULL.
/// - `foreign_keys=ON`      — SQLite does not enforce FK constraints by default.
#[derive(Debug)]
struct SqliteConnOpts;

impl CustomizeConnection<SqliteConnection, R2d2Error> for SqliteConnOpts {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), R2d2Error> {
        conn.batch_execute(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(R2d2Error::QueryError)
    }
}

/// Build a connection pool for the given SQLite database path.
///
/// Also runs any pending migrations before returning.
pub fn build_pool(database_url: &str) -> Pool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .connection_customizer(Box::new(SqliteConnOpts))
        .build(manager)
        .expect("Failed to create database connection pool");

    let mut conn = pool.get().expect("Failed to get connection for migrations");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run database migrations");

    pool
}
