// api/email_api_inbound.rs
//! API de entrada (inbound) do canal de email: recebe requisições HTTP.
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::state::EmailState;
use crate::api::email_request::NovoEmail;
use crate::api::email_response::EmailCriado;
use crate::handler::error::EmailError;

pub async fn criar(
    State(state): State<EmailState>,
    Json(novo): Json<NovoEmail>,
) -> Result<(StatusCode, Json<EmailCriado>), EmailError> {
    let destinatario = novo.destinatario.clone();
    let id = state.service.criar(novo).await?;
    Ok((
        StatusCode::CREATED,
        Json(EmailCriado { id, destinatario, status: "pendente".into() }),
    ))
}

pub async fn buscar(
    State(state): State<EmailState>,
    Path(id): Path<u64>,
) -> Result<Json<EmailCriado>, EmailError> {
    let s = state.service.buscar(id).await?.ok_or(EmailError::NaoEncontrado)?;
    Ok(Json(EmailCriado { id: s.id, destinatario: s.destinatario, status: s.status }))
}