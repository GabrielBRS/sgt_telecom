mod state;
mod api;
mod service;
mod repository;
mod handler;

pub use handler::error::EmailError;
pub use state::{EmailState, montar_email_state};
pub use service::email_service::{EmailPort, EmailService};
pub use repository::email_repository::criar_pool;  // o main chama via email::repository::criar_pool

use axum::{routing::{get, post}, Router};
use handler::handlers;

pub fn rotas() -> Router<EmailState> {
    Router::new()
        .route("/", post(handlers::criar))
        .route("/{id}", get(handlers::buscar))
}