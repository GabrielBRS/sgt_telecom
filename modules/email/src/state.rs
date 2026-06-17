use std::sync::Arc;
use sqlx::postgres::PgPool;
use crate::service::email_service::EmailPort;

#[derive(Clone)]
pub struct EmailState {
    pub service: Arc<dyn EmailPort>,
}

impl EmailState {
    pub fn new(service: Arc<dyn EmailPort>) -> Self {
        Self { service }
    }
}

/// Fábrica: monta pool → repository → service → state.
/// É o único lugar que conhece as implementações concretas do canal.
pub fn montar_email_state(pool: PgPool) -> EmailState {
    use crate::repository::email_repository::EmailRepository;
    use crate::service::email_service::EmailService;

    let repo = Arc::new(EmailRepository::new(pool));
    let service = Arc::new(EmailService::new(repo));
    EmailState::new(service)
}