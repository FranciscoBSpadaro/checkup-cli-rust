// Trait `Check` e estruturas compartilhadas (Context, Status, CheckResult)

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;

/// Status de resultado de um check
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Pass,
    Fail,
    Warning,
}

/// Resultado da execucao de um check
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Nome do check
    pub name: String,
    /// Status (Pass, Fail, Warning)
    pub status: Status,
    /// mensagem legivel
    pub message: String,
    /// Sugestao de correcao (opcional)
    pub suggestion: Option<String>,
    /// Duracao da verificacao em ms
    pub duration_ms: u128,
}

impl CheckResult {
    /// Cria um resultado de sucesso
    pub fn pass(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Pass,
            message: message.to_string(),
            suggestion: None,
            duration_ms: 0,
        }
    }

    /// Cria um resultado de falha
    pub fn fail(name: &str, message: &str, suggestion: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Fail,
            message: message.to_string(),
            suggestion: suggestion.map(String::from),
            duration_ms: 0,
        }
    }

    /// Atribui duracao ao resultado
    #[allow(dead_code)]
    pub fn with_duration(mut self, ms: u128) -> Self {
        self.duration_ms = ms;
        self
    }
}

/// Contexto injetado em cada check
pub struct Context {
    /// Caminho do projeto (root)
    pub project_path: PathBuf,
    /// Variaveis de ambiente (override para testes)
    pub env: HashMap<String, String>,
}

impl Context {
    /// Cria um novo contexto de verificacao vazio
    #[allow(dead_code)]
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            env: HashMap::new(),
        }
    }

    /// Cria contexto com overrides de ambiente (para testes)
    pub fn with_env(project_path: PathBuf, env: HashMap<String, String>) -> Self {
        Self { project_path, env }
    }
}

/// Trait que todos os checks devem implementar
#[async_trait]
#[allow(dead_code)]
pub trait Check: Send + Sync {
    /// Nome legivel do check
    fn name(&self) -> &str;

    /// retorna sugestao estatica de fix, se houver
    fn fix_suggestion(&self) -> Option<&str> {
        None
    }

    /// auto-fix: tenta corrigir o problema identificado
    /// retorna None se nao aplicavel ou None para checar o resultado
    async fn fix(&self, _ctx: &Context) -> Option<CheckResult> {
        None
    }

    /// Executa o check
    async fn run(&self, ctx: &Context) -> CheckResult;
}

// Re-export dos checks individuais
pub mod command;
pub mod envfile;
pub mod envvar;
pub mod port;
pub mod service;
pub mod version;

pub use command::CommandCheck;
pub use envfile::EnvFileCheck;
pub use envvar::EnvVarCheck;
pub use port::PortCheck;
pub use service::ServiceCheck;
pub use version::VersionCheck;
