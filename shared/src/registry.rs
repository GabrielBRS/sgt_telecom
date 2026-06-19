// shared/src/registry.rs
use axum::Router;
use sqlx::postgres::PgPool;

/// Dependências compartilhadas entregues a cada módulo pra ele se montar.
/// Hoje só o pool; amanhã ganha cliente de GPU, cache, etc. Cresce raramente.
#[derive(Clone)]
pub struct Infra {
    pub pool: PgPool,
}

/// O REGISTRO de um módulo. É a "anotação" que a folha expõe pra cima.
/// `construir` recebe a Infra e devolve o Router JÁ com o state embutido (Router<()>).
pub struct RegistroModulo {
    pub nome: &'static str,
    pub prefixo: &'static str,
    pub construir: fn(&Infra) -> Router,
}