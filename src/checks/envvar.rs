// Check de variavel de ambiente: verifica se vars estao definidas
// Usa std::env::var para cada variavel requerida, com ctx.env como override

use super::{Check, CheckResult, Context};
use crate::config::EnvConfig;

/// Check que verifica variaveis de ambiente
pub struct EnvVarCheck {
    /// Nome legivel do grupo
    name: String,
    /// Variaveis que devem estar definidas
    required: Vec<String>,
}

impl EnvVarCheck {
    /// Cria um novo check de variaveis de ambiente
    pub fn new(name: &str, config: &EnvConfig) -> Self {
        Self {
            name: name.to_string(),
            required: config.required.clone(),
        }
    }

    /// Resolve uma variavel: primeiro no ctx.env (overrides), depois no ambiente real
    fn resolve_env(ctx: &Context, key: &str) -> Option<String> {
        // ctx.env tem prioridade (permite injecao em testes)
        ctx.env
            .get(key)
            .cloned()
            .or_else(|| std::env::var(key).ok())
    }
}

#[async_trait::async_trait]
impl Check for EnvVarCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn fix_suggestion(&self) -> Option<&str> {
        Some("set the missing environment variables")
    }

    async fn fix(&self, ctx: &Context) -> Option<CheckResult> {
        let missing: Vec<&str> = self
            .required
            .iter()
            .filter(|key| Self::resolve_env(ctx, key).is_none())
            .map(|s| s.as_str())
            .collect();

        if missing.is_empty() {
            return Some(CheckResult::pass(
                &self.name,
                "all environment variables already set",
            ));
        }

        // Tenta carregar do .env file se existir
        let envfile_path = ctx.project_path.join(".env");
        if envfile_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&envfile_path) {
                for line in content.lines() {
                    let line = line.trim();
                    if line.is_empty() || line.starts_with('#') {
                        continue;
                    }
                    if let Some((key, value)) = line.split_once('=') {
                        if missing.contains(&key.trim()) {
                            std::env::set_var(key.trim(), value.trim());
                        }
                    }
                }
            }
        }

        // Verifica novamente se todas foram resolvidas
        let still_missing: Vec<&str> = self
            .required
            .iter()
            .filter(|key| Self::resolve_env(ctx, key).is_none())
            .map(|s| s.as_str())
            .collect();

        if still_missing.is_empty() {
            return Some(CheckResult::pass(
                &self.name,
                &format!("loaded missing vars from .env: {}", missing.join(", ")),
            ));
        }

        Some(CheckResult::fail(
            &self.name,
            &format!(
                "cannot auto-fix: still missing {}",
                still_missing.join(", ")
            ),
            Some("set the missing environment variables manually"),
        ))
    }

    async fn run(&self, ctx: &Context) -> CheckResult {
        let missing: Vec<&str> = self
            .required
            .iter()
            .filter(|key| Self::resolve_env(ctx, key).is_none())
            .map(|s| s.as_str())
            .collect();

        if missing.is_empty() {
            CheckResult::pass(
                &self.name,
                &format!("all {} environment variables set", self.required.len()),
            )
        } else {
            CheckResult::fail(
                &self.name,
                &format!(
                    "missing {} env var(s): {}",
                    missing.len(),
                    missing.join(", ")
                ),
                Some("set the missing environment variables"),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_envvar_all_set() {
        let config = EnvConfig {
            required: vec!["HOME".to_string()],
        };
        let check = EnvVarCheck::new("Environment", &config);
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Pass);
    }

    #[tokio::test]
    async fn test_envvar_missing() {
        let config = EnvConfig {
            required: vec!["VARIAVEL_INEXISTENTE_XYZ_12345".to_string()],
        };
        let check = EnvVarCheck::new("Environment", &config);

        let mut env = HashMap::new();
        env.insert("HOME".to_string(), "/root".to_string());
        let ctx = Context {
            project_path: std::env::current_dir().unwrap(),
            env,
        };
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Fail);
        assert!(result.message.contains("VARIAVEL_INEXISTENTE_XYZ_12345"));
    }

    #[tokio::test]
    async fn test_envvar_resolves_from_ctx_with_fallback() {
        let config = EnvConfig {
            required: vec!["MY_CUSTOM_VAR".to_string()],
        };
        let check = EnvVarCheck::new("Custom", &config);

        let mut env = HashMap::new();
        env.insert("MY_CUSTOM_VAR".to_string(), "custom_value".to_string());
        let ctx = Context {
            project_path: std::env::current_dir().unwrap(),
            env,
        };
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Pass);
    }
}
