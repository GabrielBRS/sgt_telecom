use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct NovoEmail {
    pub destinatario: String,
    pub assunto: String,
}

#[derive(Debug, Serialize)]
pub struct EmailCriado {
    pub id: u64,
    pub destinatario: String,
    pub status: String,
}