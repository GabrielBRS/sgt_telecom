use std::env;
use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("variável de ambiente obrigatória ausente: {0}")]
    Faltando(&'static str),

    #[error("valor inválido para {chave}: {valor} ({motivo})")]
    Invalido {
        chave: &'static str,
        valor: String,
        motivo: String,
    },

    #[error("falha ao ler arquivo de config {path}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("TOML inválido")]
    Toml(#[from] toml::de::Error),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SslMode {
    Disable,
    Require,
}

impl Default for SslMode {
    fn default() -> Self {
        SslMode::Disable
    }
}

fn default_port() -> u16 {
    5432
}
fn default_max_connections() -> u32 {
    10
}
fn default_connect_timeout_secs() -> u64 {
    5
}

#[derive(Debug, Deserialize)]
struct FileRoot {
    postgres: PostgresConfigRaw,
}

#[derive(Debug, Deserialize)]
struct PostgresConfigRaw {
    host: String,
    #[serde(default = "default_port")]
    port: u16,
    user: String,
    password: String,
    database: String,
    #[serde(default = "default_max_connections")]
    max_connections: u32,
    #[serde(default = "default_connect_timeout_secs")]
    connect_timeout_secs: u64,
    #[serde(default)]
    ssl_mode: SslMode,
}

impl From<PostgresConfigRaw> for PostgresConfig {
    fn from(r: PostgresConfigRaw) -> Self {
        Self {
            host: r.host,
            port: r.port,
            user: r.user,
            password: r.password,
            database: r.database,
            max_connections: r.max_connections,
            connect_timeout: Duration::from_secs(r.connect_timeout_secs),
            ssl_mode: r.ssl_mode,
        }
    }
}

impl PostgresConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            host: req("PG_HOST")?,
            port: parse_or("PG_PORT", 5432)?,
            user: req("PG_USER")?,
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

    pub fn from_toml_file(path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        let texto = std::fs::read_to_string(path).map_err(|e| ConfigError::Io {
            path: path.display().to_string(),
            source: e,
        })?;
        Self::from_toml_str(&texto)
    }

    pub fn from_toml_str(s: &str) -> Result<Self, ConfigError> {
        let root: FileRoot = toml::from_str(s)?;
        Ok(root.postgres.into())
    }

    pub fn from_toml_file_with_env_override(
        path: impl AsRef<Path>,
    ) -> Result<Self, ConfigError> {
        let mut cfg = Self::from_toml_file(path)?;

        if let Some(v) = opt("PG_HOST") {
            cfg.host = v;
        }
        if let Some(v) = opt("PG_USER") {
            cfg.user = v;
        }
        if let Some(v) = opt("PG_PASSWORD") {
            cfg.password = v;
        }
        if let Some(v) = opt("PG_DATABASE") {
            cfg.database = v;
        }
        cfg.port = parse_or("PG_PORT", cfg.port)?;
        cfg.max_connections = parse_or("PG_MAX_CONNECTIONS", cfg.max_connections)?;
        cfg.connect_timeout =
            Duration::from_secs(parse_or("PG_CONNECT_TIMEOUT_SECS", cfg.connect_timeout.as_secs())?);
        if let Some(v) = opt("PG_SSL_MODE") {
            cfg.ssl_mode = match v.as_str() {
                "require" => SslMode::Require,
                _ => SslMode::Disable,
            };
        }

        Ok(cfg)
    }

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

    pub fn safe_display(&self) -> String {
        format!(
            "postgres://{}:***@{}:{}/{} (max={}, ssl={:?})",
            self.user, self.host, self.port, self.database, self.max_connections, self.ssl_mode
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
            host: "h".into(),
            port: 5432,
            user: "u".into(),
            password: "supersecreta".into(),
            database: "d".into(),
            max_connections: 5,
            connect_timeout: Duration::from_secs(5),
            ssl_mode: SslMode::Disable,
        };
        assert!(!cfg.safe_display().contains("supersecreta"));
    }

    #[test]
    fn from_toml_str_parseia_completo() {
        let toml = r#"
            [postgres]
            host = "192.168.15.200"
            port = 5432
            user = "sgt"
            password = "secret"
            database = "sgt_maisclinical"
            max_connections = 20
            connect_timeout_secs = 8
            ssl_mode = "require"
        "#;
        let cfg = PostgresConfig::from_toml_str(toml).unwrap();
        assert_eq!(cfg.host, "192.168.15.200");
        assert_eq!(cfg.max_connections, 20);
        assert_eq!(cfg.connect_timeout, Duration::from_secs(8));
        assert_eq!(cfg.ssl_mode, SslMode::Require);
    }

    #[test]
    fn from_toml_str_aplica_defaults() {
        let toml = r#"
            [postgres]
            host = "h"
            user = "u"
            password = "p"
            database = "d"
        "#;
        let cfg = PostgresConfig::from_toml_str(toml).unwrap();
        assert_eq!(cfg.port, 5432);
        assert_eq!(cfg.max_connections, 10);
        assert_eq!(cfg.connect_timeout, Duration::from_secs(5));
        assert_eq!(cfg.ssl_mode, SslMode::Disable);
    }

    #[test]
    fn from_toml_str_falha_campo_obrigatorio() {
        let toml = r#"
            [postgres]
            user = "u"
            password = "p"
            database = "d"
        "#;
        assert!(PostgresConfig::from_toml_str(toml).is_err());
    }
}