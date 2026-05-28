// Check de variavel de ambiente: verifica se vars estao definidas
// Usa std::env::var para cada variavel requerida

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
}

#[async_trait::async_trait]
impl Check for EnvVarCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, ctx: &Context) -> CheckResult {
        let missing: Vec<&str> = self
            .required
            .iter()
            .filter(|key| !ctx.env.contains_key(*key))
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
            required: vec!["VARIAVEL_INEXISTENTE_XYZ".to_string()],
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
        assert!(result.message.contains("VARIAVEL_INEXISTENTE_XYZ"));
    }
}
