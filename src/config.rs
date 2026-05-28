// Modulo de configuracao: parsing do .checkup.toml
// Usa serde para desserializacao e thiserror para erros tipados

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Erros que podem ocorrer ao carregar a configuracao
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("arquivo de configuracao nao encontrado: {0}")]
    NotFound(PathBuf),

    #[error("erro ao ler arquivo de configuracao: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("erro ao parsear TOML: {0}")]
    ParseError(#[from] toml::de::Error),
}

/// Configuracao completa do .checkup.toml
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct CheckupConfig {
    /// Checks de comando (verifica se comando existe no PATH)
    #[serde(default)]
    pub commands: HashMap<String, CommandConfig>,

    /// Checks de versao (verifica versao de ferramentas)
    #[serde(default)]
    pub versions: HashMap<String, VersionConfig>,

    /// Checks de servico (verifica se porta TCP esta respondendo)
    #[serde(default)]
    pub services: HashMap<String, ServiceConfig>,

    /// Portas que devem estar livres
    #[serde(default)]
    pub ports: PortsConfig,

    /// Variaveis de ambiente requeridas
    #[serde(default)]
    pub env: EnvConfig,

    /// Arquivo .env
    #[serde(default)]
    pub envfile: EnvFileConfig,
}

/// Configuracao de um check de comando
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CommandConfig {
    /// Comando a ser buscado no PATH
    pub command: String,
    /// Opcional: sugestao de instalacao
    pub fix: Option<String>,
}

/// Configuracao de um check de versao
#[derive(Debug, Clone, serde::Deserialize)]
pub struct VersionConfig {
    /// Comando para obter a versao (ex: "--version")
    #[serde(default = "default_version_flag")]
    pub version_flag: String,
    /// Versao esperada no formato semver constraint (ex: ">=20")
    pub expected: String,
    /// Opcional: sugestao de correcao
    pub fix: Option<String>,
}

fn default_version_flag() -> String {
    "--version".to_string()
}

/// Configuracao de um check de servico (TCP port)
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ServiceConfig {
    /// Porta TCP que deve estar respondendo
    pub port: u16,
    /// Host para verificar (padrao: localhost)
    #[serde(default = "default_host")]
    pub host: String,
}

fn default_host() -> String {
    "localhost".to_string()
}

/// Configuracao de portas livres
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct PortsConfig {
    /// Lista de portas que devem estar livres
    #[serde(default)]
    pub free: Vec<u16>,
}

/// Configuracao de variaveis de ambiente
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct EnvConfig {
    /// Lista de variaveis que devem estar definidas
    #[serde(default)]
    pub required: Vec<String>,
}

/// Configuracao do arquivo .env
#[derive(Debug, Clone, serde::Deserialize, Default)]
pub struct EnvFileConfig {
    /// Caminho do arquivo .env (padrao: .env)
    #[serde(default = "default_env_path")]
    pub path: String,

    /// Lista de chaves que devem existir no .env
    #[serde(default)]
    pub required: Vec<String>,
}

fn default_env_path() -> String {
    ".env".to_string()
}

impl CheckupConfig {
    /// Carrega a configuracao a partir de um caminho de arquivo
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(ConfigError::NotFound(path.to_path_buf()));
        }

        let content = fs::read_to_string(path)?;
        let config: CheckupConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Carrega a configuracao do diretorio do projeto
    pub fn load_from_dir<P: AsRef<Path>>(dir: P) -> Result<Self, ConfigError> {
        Self::load(dir.as_ref().join(".checkup.toml"))
    }

    /// Cria uma configuracao padrao (para `checkup init`)
    pub fn default_config() -> Self {
        let mut commands = HashMap::new();
        commands.insert(
            "node".to_string(),
            CommandConfig {
                command: "node".to_string(),
                fix: Some("curl -fsSL https://fnm.install | bash".to_string()),
            },
        );

        let mut versions = HashMap::new();
        versions.insert(
            "Node.js".to_string(),
            VersionConfig {
                version_flag: "--version".to_string(),
                expected: ">=20".to_string(),
                fix: Some("fnm install 20".to_string()),
            },
        );

        Self {
            commands,
            versions,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_minimal_config() {
        let toml_str = r#"
[commands.node]
command = "node"
fix = "fnm install"
"#;
        let config: CheckupConfig = toml::from_str(toml_str).unwrap();
        assert!(config.commands.contains_key("node"));
        assert_eq!(config.commands["node"].command, "node");
    }

    #[test]
    fn test_parse_full_config() {
        let toml_str = r#"
[commands.node]
command = "node"

[versions.node]
version_flag = "--version"
expected = ">=20"

[services.postgres]
port = 5432

[ports]
free = [3000, 8080]

[env]
required = ["DATABASE_URL", "JWT_SECRET"]

[envfile]
path = ".env"
required = ["DATABASE_URL"]
"#;
        let config: CheckupConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.commands.len(), 1);
        assert_eq!(config.versions.len(), 1);
        assert_eq!(config.services.len(), 1);
        assert_eq!(config.ports.free, vec![3000, 8080]);
        assert_eq!(config.env.required.len(), 2);
        assert_eq!(config.envfile.required, vec!["DATABASE_URL"]);
    }

    #[test]
    fn test_default_config() {
        let config = CheckupConfig::default_config();
        assert!(config.commands.contains_key("node"));
    }

    #[test]
    fn test_config_not_found() {
        let result = CheckupConfig::load("/caminho/inexistente/.checkup.toml");
        assert!(matches!(result, Err(ConfigError::NotFound(_))));
    }
}
