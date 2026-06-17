use std::env;
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("variável de ambiente obrigatória ausente: {0}")]
    Faltando(&'static str),
    #[error("valor inválido para {chave}: {valor} ({motivo})")]
    Invalido { chave: &'static str, valor: String, motivo: String },
}

#[derive(Debug, Clone)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub connect_timeout: Duration,
    pub ssl_mode: SslMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SslMode {
    Disable,
    Require,
}

impl PostgresConfig {
    /// Carrega da env. Falha cedo se algo obrigatório faltar ou for inválido.
    /// Convenção: prefixo PG_ (PG_HOST, PG_PORT, ...).
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            host:     req("PG_HOST")?,
            port:     parse_or("PG_PORT", 5432)?,
            user:     req("PG_USER")?,
            password: req("PG_PASSWORD")?,
            database: req("PG_DATABASE")?,
            max_connections: parse_or("PG_MAX_CONNECTIONS", 10)?,
            connect_timeout: Duration::from_secs(parse_or("PG_CONNECT_TIMEOUT_SECS", 5)?),
            ssl_mode: match opt("PG_SSL_MODE").as_deref() {
                Some("require") => SslMode::Require,
                _ => SslMode::Disable,
            },
        })
    }

    /// Connection string no formato libpq/SQLx.
    /// Não logue isto: contém a senha.
    pub fn connection_string(&self) -> String {
        let ssl = match self.ssl_mode {
            SslMode::Require => "require",
            SslMode::Disable => "disable",
        };
        format!(
            "postgres://{}:{}@{}:{}/{}?sslmode={}",
            self.user, self.password, self.host, self.port, self.database, ssl
        )
    }

    /// Versão segura pra log: mascara a senha.
    pub fn safe_display(&self) -> String {
        format!(
            "postgres://{}:***@{}:{}/{} (max={}, ssl={:?})",
            self.user, self.host, self.port, self.database,
            self.max_connections, self.ssl_mode
        )
    }
}

fn req(chave: &'static str) -> Result<String, ConfigError> {
    env::var(chave).map_err(|_| ConfigError::Faltando(chave))
}

fn opt(chave: &str) -> Option<String> {
    env::var(chave).ok()
}

fn parse_or<T: std::str::FromStr>(chave: &'static str, default: T) -> Result<T, ConfigError>
where
    T::Err: std::fmt::Display,
{
    match env::var(chave) {
        Ok(v) => v.parse::<T>().map_err(|e| ConfigError::Invalido {
            chave,
            valor: v,
            motivo: e.to_string(),
        }),
        Err(_) => Ok(default),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_string_formata_correto() {
        let cfg = PostgresConfig {
            host: "192.168.15.201".into(),
            port: 5432,
            user: "sgt".into(),
            password: "secret".into(),
            database: "telecom".into(),
            max_connections: 10,
            connect_timeout: Duration::from_secs(5),
            ssl_mode: SslMode::Disable,
        };
        assert_eq!(
            cfg.connection_string(),
            "postgres://sgt:secret@192.168.15.201:5432/telecom?sslmode=disable"
        );
    }

    #[test]
    fn safe_display_mascara_senha() {
        let cfg = PostgresConfig {
            host: "h".into(), port: 5432, user: "u".into(),
            password: "supersecreta".into(), database: "d".into(),
            max_connections: 5, connect_timeout: Duration::from_secs(5),
            ssl_mode: SslMode::Disable,
        };
        assert!(!cfg.safe_display().contains("supersecreta"));
    }
}