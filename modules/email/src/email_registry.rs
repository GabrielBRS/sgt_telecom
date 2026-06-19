// modules/email/src/email_registry.rs
use axum::Router;
use shared::registry::{Infra, RegistroModulo};

/// O módulo de email se declara aqui. Esta função é o "index" do canal.
pub fn registrar() -> RegistroModulo {
    RegistroModulo {
        nome: "email",
        prefixo: "/emails",
        construir,
    }
}

/// Toda a injeção de dependência do canal acontece AQUI (na folha):
/// pool → repository → service → state, e o state entra no router.
/// Sai um Router<()> autossuficiente — o registry de cima nem vê EmailState.
fn construir(infra: &Infra) -> Router {
    let state = crate::montar_email_state(infra.pool.clone());
    crate::rotas().with_state(state)
}