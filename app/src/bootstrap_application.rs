// app/src/bootstrap_application.rs
use std::net::SocketAddr;

use anyhow::Context as _;
use axum::Router;
use config::PostgresConfig;
use shared::registry::Infra;

pub struct Application {
    router: Router,
    infra: Infra,
    addr: SocketAddr,
}

impl Application {
    /// COMPOSITION ROOT: resolve o grafo inteiro em startup time, fail-fast.
    pub async fn build() -> anyhow::Result<Self> {
        // 1. config (falha cedo; senha mascarada no log)
        let cfg = PostgresConfig::from_toml_file_with_env_override("config/postgres.toml")
            .context("falha ao carregar config Postgres")?;
        tracing::info!(pg = %cfg.safe_display(), "config carregada");

        // 2. pool único compartilhado — criado UMA vez aqui
        let pool = shared::db::criar_pool(&cfg)
            .await
            .context("falha ao abrir pool Postgres")?;

        // 3. infra entregue a cada módulo pra ele se montar
        let infra = Infra { pool };

        // 4. INTEGRA: itera o índice de módulos. Este laço NÃO cresce com o nº de canais.
        let mut router = Router::new();
        for reg in crate::app_registry::registrar() {
            tracing::info!(modulo = reg.nome, prefixo = reg.prefixo, "registrando módulo");
            router = router.nest(reg.prefixo, (reg.construir)(&infra));
        }

        let addr = std::env::var("SGT_BIND")
            .unwrap_or_else(|_| "0.0.0.0:3000".into())
            .parse()
            .context("SGT_BIND inválido")?;

        Ok(Self { router, infra, addr })
    }

    /// Ciclo de vida: serve + graceful shutdown + dreno do pool.
    pub async fn run(self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(self.addr)
            .await
            .with_context(|| format!("falha ao bindar {}", self.addr))?;
        tracing::info!(addr = %self.addr, "servidor pronto");

        axum::serve(listener, self.router)
            .with_graceful_shutdown(shutdown_signal())
            .await
            .context("erro no servidor HTTP")?;

        // ShutdownEvent: dreno ordenado
        tracing::info!("fechando pool Postgres");
        self.infra.pool.close().await;
        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.expect("handler de Ctrl+C");
    };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("handler de SIGTERM")
            .recv()
            .await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! { _ = ctrl_c => {}, _ = terminate => {} }
    tracing::info!("sinal de shutdown recebido");
}