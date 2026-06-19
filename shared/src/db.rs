// shared/src/db.rs
use config::PostgresConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};

/// Pool único compartilhado por todos os canais (ApplicationScoped).
/// Criado UMA vez, no Application::build().
pub async fn criar_pool(cfg: &PostgresConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(cfg.max_connections)   // campo
        .acquire_timeout(cfg.connect_timeout)   // campo, já é Duration
        .connect(&cfg.connection_string())      // método
        .await
}

/// Readiness check — pra um futuro /q/health/ready.
pub async fn ping(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}