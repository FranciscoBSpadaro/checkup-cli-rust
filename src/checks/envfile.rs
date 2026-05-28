// Check de arquivo .env: verifica se .env existe e contem variaveis requeridas
// Parse linha a linha no formato CHAVE=VALOR

use super::{Check, CheckResult, Context};
use crate::config::EnvFileConfig;
use std::fs;
use std::path::PathBuf;

/// Check que verifica o arquivo .env
pub struct EnvFileCheck {
    /// Caminho do arquivo .env relativo ao projeto
    path: PathBuf,
    /// Chaves que devem existir no arquivo
    required_keys: Vec<String>,
}

impl EnvFileCheck {
    /// Cria um novo check de envfile a partir da configuracao
    pub fn new(config: &EnvFileConfig) -> Self {
        Self {
            path: PathBuf::from(&config.path),
            required_keys: config.required.clone(),
        }
    }
}

#[async_trait::async_trait]
impl Check for EnvFileCheck {
    fn name(&self) -> &str {
        ".env"
    }

    async fn run(&self, ctx: &Context) -> CheckResult {
        let full_path = ctx.project_path.join(&self.path);

        // Verifica se o arquivo existe
        if !full_path.exists() {
            let missing = self.required_keys.join(", ");
            return CheckResult::fail(
                ".env",
                &format!(
                    "file '{}' not found (missing: {})",
                    self.path.display(),
                    missing
                ),
                Some(&format!("create {} from .env.example", self.path.display())),
            );
        }

        // Le o arquivo e extrai as chaves
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(e) => {
                return CheckResult::fail(
                    ".env",
                    &format!("error reading '{}': {}", self.path.display(), e),
                    None,
                );
            }
        };

        let found_keys: Vec<&str> = content
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    return None;
                }
                line.split('=').next().map(|k| k.trim())
            })
            .collect();

        // Verifica quais chaves estao faltando
        let missing: Vec<&str> = self
            .required_keys
            .iter()
            .filter(|key| !found_keys.contains(&key.as_str()))
            .map(|s| s.as_str())
            .collect();

        if missing.is_empty() {
            CheckResult::pass(
                ".env",
                &format!(
                    "all {} required keys present in '{}'",
                    self.required_keys.len(),
                    self.path.display()
                ),
            )
        } else {
            CheckResult::fail(
                ".env",
                &format!(
                    "missing {} key(s) in '{}': {}",
                    missing.len(),
                    self.path.display(),
                    missing.join(", ")
                ),
                Some(&format!("add the missing keys to {}", self.path.display())),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_envfile_all_keys_present() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();
        writeln!(file, "JWT_SECRET=abc123").unwrap();

        let config = EnvFileConfig {
            path: file.path().to_str().unwrap().to_string(),
            required: vec!["DATABASE_URL".to_string(), "JWT_SECRET".to_string()],
        };

        let check = EnvFileCheck::new(&config);
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Pass);
    }

    #[tokio::test]
    async fn test_envfile_missing_keys() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "DATABASE_URL=postgres://localhost/db").unwrap();

        let config = EnvFileConfig {
            path: file.path().to_str().unwrap().to_string(),
            required: vec!["DATABASE_URL".to_string(), "JWT_SECRET".to_string()],
        };

        let check = EnvFileCheck::new(&config);
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Fail);
        assert!(result.message.contains("JWT_SECRET"));
    }

    #[tokio::test]
    async fn test_envfile_not_found() {
        let config = EnvFileConfig {
            path: "/tmp/arquivo_inexistente_12345.env".to_string(),
            required: vec!["DATABASE_URL".to_string()],
        };

        let check = EnvFileCheck::new(&config);
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Fail);
    }
}
