use std::sync::Arc;
use async_trait::async_trait;
use crate::api::email_request::NovoEmail;
use crate::handler::error::EmailError;

// ---- PORTA de saída (o repository implementa) ----
#[async_trait]
pub trait Repositorio: Send + Sync {
    async fn salvar(&self, reg: &RegistroEmail) -> Result<u64, EmailError>;
    async fn buscar(&self, id: u64) -> Result<Option<StatusEmail>, EmailError>;
}

#[derive(Debug, Clone)]
pub struct RegistroEmail {
    pub destinatario: String,
    pub assunto: String,
    pub status: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone)]
pub struct StatusEmail {
    pub id: u64,
    pub destinatario: String,
    pub status: String,
}

// ---- PORTA de entrada (o handler chama) ----
#[async_trait]
pub trait EmailPort: Send + Sync {
    async fn criar(&self, novo: NovoEmail) -> Result<u64, EmailError>;
    async fn buscar(&self, id: u64) -> Result<Option<StatusEmail>, EmailError>;
}

// ---- Caso de uso ----
pub struct EmailService {
    repo: Arc<dyn Repositorio>,
}

impl EmailService {
    pub fn new(repo: Arc<dyn Repositorio>) -> Self {
        Self { repo }
    }
}

#[async_trait]
impl EmailPort for EmailService {
    async fn criar(&self, novo: NovoEmail) -> Result<u64, EmailError> {
        if novo.destinatario.trim().is_empty() {
            return Err(EmailError::Validacao("destinatário vazio".into()));
        }
        // idempotency_key: por ora derivada do conteúdo; depois pode vir do header
        let idempotency_key = format!("{}:{}", novo.destinatario, novo.assunto);

        let reg = RegistroEmail {
            destinatario: novo.destinatario.clone(),
            assunto: novo.assunto,
            status: "pendente".into(),
            idempotency_key,
        };
        let id = self.repo.salvar(&reg).await?;
        tracing::info!(id, destinatario = %novo.destinatario, "email persistido");
        // TODO: aqui depois entram validador/renderizador/transporte/eventos
        Ok(id)
    }

    async fn buscar(&self, id: u64) -> Result<Option<StatusEmail>, EmailError> {
        self.repo.buscar(id).await
    }
}