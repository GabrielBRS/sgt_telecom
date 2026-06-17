use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions};
use config::PostgresConfig;
use crate::handler::error::EmailError;
use crate::service::email_service::{Repositorio, RegistroEmail, StatusEmail};

/// Cria o pool a partir da config. Chamado UMA vez no main.
/// Cria o pool a partir da config. Chamado UMA vez no main.
pub async fn criar_pool(cfg: &PostgresConfig) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .acquire_timeout(cfg.connect_timeout)
        .connect(&cfg.connection_string())
        .await?;
    Ok(pool)
}

/// Adaptador de saída: implementa a porta Repositorio falando com Postgres.
pub struct EmailRepository {
    pool: PgPool,
}

impl EmailRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repositorio for EmailRepository {
    async fn salvar(&self, reg: &RegistroEmail) -> Result<u64, EmailError> {
        let row: (i64,) = sqlx::query_as(
            "INSERT INTO emails (destinatario, assunto, status, idempotency_key)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (idempotency_key) DO UPDATE SET updated_at = now()
             RETURNING id",
        )
            .bind(&reg.destinatario)
            .bind(&reg.assunto)
            .bind(&reg.status)
            .bind(&reg.idempotency_key)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| EmailError::Interno(format!("falha ao salvar: {e}")))?;

        Ok(row.0 as u64)
    }

    async fn buscar(&self, id: u64) -> Result<Option<StatusEmail>, EmailError> {
        let row: Option<(i64, String, String)> = sqlx::query_as(
            "SELECT id, destinatario, status FROM emails WHERE id = $1",
        )
            .bind(id as i64)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| EmailError::Interno(format!("falha ao buscar: {e}")))?;

        Ok(row.map(|(id, destinatario, status)| StatusEmail {
            id: id as u64,
            destinatario,
            status,
        }))
    }
}