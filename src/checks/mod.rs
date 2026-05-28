// Modulo checks: define a trait Check (plugin system)
// Cada verificacao implementa esta trait

use std::collections::HashMap;
use std::path::PathBuf;

// Submodulos de checks
pub mod command;
pub mod envfile;
pub mod envvar;
pub mod port;
pub mod service;
pub mod version;

// Re-export das structs principais
pub use command::CommandCheck;
pub use envfile::EnvFileCheck;
pub use envvar::EnvVarCheck;
pub use port::PortCheck;
pub use service::ServiceCheck;
pub use version::VersionCheck;

/// Status de resultado de um check
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    /// Check passou sem problemas
    Pass,
    /// Check falhou — algo esta errado no ambiente
    Fail,
    /// Check passou com alerta — atencao necessaria
    Warning,
    /// Check foi pulado — nao aplicavel ao ambiente atual
    Skipped,
}

/// Resultado da execucao de um check
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Nome legivel do check (ex: "Node.js")
    pub name: String,
    /// Status do resultado
    pub status: Status,
    /// Mensagem descritiva (ex: "found 18.2, expected >= 20")
    pub message: String,
    /// Sugestao de comando para correcao manual
    pub fix_suggestion: Option<String>,
}

impl CheckResult {
    /// Cria um resultado de sucesso
    pub fn pass(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Pass,
            message: message.to_string(),
            fix_suggestion: None,
        }
    }

    /// Cria um resultado de falha com sugestao de correcao
    pub fn fail(name: &str, message: &str, fix: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Fail,
            message: message.to_string(),
            fix_suggestion: fix.map(|s| s.to_string()),
        }
    }

    /// Cria um resultado de warning
    pub fn warn(name: &str, message: &str) -> Self {
        Self {
            name: name.to_string(),
            status: Status::Warning,
            message: message.to_string(),
            fix_suggestion: None,
        }
    }
}

/// Contexto compartilhado entre todos os checks
#[derive(Debug, Clone)]
pub struct Context {
    /// Caminho raiz do projeto (onde esta o .checkup.toml)
    pub project_path: PathBuf,
    /// Variaveis de ambiente atuais
    pub env: HashMap<String, String>,
}

impl Context {
    /// Cria um novo contexto a partir do caminho do projeto
    pub fn new(project_path: PathBuf) -> Self {
        let env: HashMap<String, String> = std::env::vars().collect();
        Self { project_path, env }
    }
}

/// Trait que cada check deve implementar.
/// A trait e Send + Sync para permitir uso em paralelo com tokio.
#[async_trait::async_trait]
pub trait Check: Send + Sync {
    /// Nome legivel do check para exibicao no output
    fn name(&self) -> &str;

    /// Executa o check e retorna o resultado
    async fn run(&self, ctx: &Context) -> CheckResult;

    /// Tenta corrigir automaticamente o problema encontrado.
    /// Retorna None se o check nao suporta auto-fix.
    async fn fix(&self, _ctx: &Context) -> Option<CheckResult> {
        None
    }

    /// Comando sugerido para correcao manual quando auto-fix nao for possivel
    fn fix_suggestion(&self) -> Option<&str> {
        None
    }
}
