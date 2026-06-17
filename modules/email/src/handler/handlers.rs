use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use crate::state::EmailState;
use crate::api::email_request::{EmailCriado, NovoEmail};
use crate::handler::error::EmailError;

pub async fn criar(
    State(state): State<EmailState>,   // State SEMPRE antes do Json
    Json(novo): Json<NovoEmail>,       // Json por último: consome o body
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