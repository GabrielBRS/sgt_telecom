use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Debug)]
pub enum EmailError {
    Validacao(String),
    NaoEncontrado,
    Interno(String),
}

#[derive(Serialize)]
struct CorpoErro {
    erro: String,
    detalhe: String,
}

impl IntoResponse for EmailError {
    fn into_response(self) -> Response {
        let (status, erro, detalhe) = match self {
            EmailError::Validacao(d) => (StatusCode::BAD_REQUEST, "validacao", d),
            EmailError::NaoEncontrado => {
                (StatusCode::NOT_FOUND, "nao_encontrado", "recurso inexistente".into())
            }
            EmailError::Interno(d) => {
                tracing::error!(detalhe = %d, "erro interno");
                // não vaza detalhe interno pro cliente
                (StatusCode::INTERNAL_SERVER_ERROR, "interno", "erro interno".into())
            }
        };
        (status, Json(CorpoErro { erro: erro.into(), detalhe })).into_response()
    }
}