use axum::Router;
use config::PostgresConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // 1. carrega config — falha cedo se faltar env
    let pg_cfg = PostgresConfig::from_env()?;
    tracing::info!("conectando: {}", pg_cfg.safe_display());  // mascarado!

    // 2. abre o pool UMA vez (compartilhado entre todos os canais)
    let pool = email::criar_pool(&pg_cfg).await?;

    // 3. injeta pool → repository → service → state
    let email_state = email::montar_email_state(pool);

    let app = Router::new()
        .nest("/emails", email::rotas())
        .with_state(email_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("servidor em http://localhost:3000");
    axum::serve(listener, app).await?;
    Ok(())
}