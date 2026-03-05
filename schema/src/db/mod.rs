//! Database layer — SQLx / PostgreSQL.
//!
//! This module exposes:
//!
//! * **`types`** – [`sqlx::Type`]-derived Rust enums that mirror the
//!   PostgreSQL `CREATE TYPE` declarations in the migration.
//! * **`models`** – [`sqlx::FromRow`] structs, one per database table.
//! * **[`migrate`]** – runs all pending migrations against a live pool.
//!
//! # Quick-start
//!
//! ```rust,no_run
//! use sqlx::PgPool;
//! use schema::db;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let pool = PgPool::connect("postgres://localhost/pbv2").await?;
//!     db::migrate(&pool).await?;
//!     Ok(())
//! }
//! ```

pub mod models;
pub mod types;

pub use models::*;
pub use types::*;

/// Run all pending migrations from the `migrations/` directory at the
/// workspace root.
///
/// Internally this calls [`sqlx::migrate!`] which embeds the SQL files at
/// compile time, so no filesystem access is required at runtime.
pub async fn migrate(pool: &sqlx::PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("../migrations")
        .run(pool)
        .await
        .map_err(anyhow::Error::from)
}
