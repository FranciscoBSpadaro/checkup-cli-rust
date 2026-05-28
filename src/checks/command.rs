// Check de comando: verifica se um comando existe no PATH
// Utiliza `which` crate para busca cross-platform (funciona em Linux, Mac e Windows)

use super::{Check, CheckResult, Context};

/// Configuracao para o check de comando
pub struct CommandCheck {
    /// Nome legivel do comando (ex: "Node.js")
    name: String,
    /// Comando a ser buscado no PATH (ex: "node")
    command: String,
    /// Sugestao de instalacao caso o comando nao seja encontrado
    fix_suggestion: Option<String>,
}

impl CommandCheck {
    /// Cria um novo check de comando
    pub fn new(name: &str, command: &str) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            fix_suggestion: None,
        }
    }

    /// Define a sugestao de correcao
    pub fn with_fix(mut self, suggestion: &str) -> Self {
        self.fix_suggestion = Some(suggestion.to_string());
        self
    }
}

#[async_trait::async_trait]
impl Check for CommandCheck {
    fn name(&self) -> &str {
        &self.name
    }

    async fn run(&self, _ctx: &Context) -> CheckResult {
        // Usa `which` crate para busca cross-platform
        match which::which(&self.command) {
            Ok(path) => CheckResult::pass(&self.name, &format!("found at {}", path.display())),
            Err(_) => CheckResult::fail(
                &self.name,
                &format!("command '{}' not found in PATH", self.command),
                self.fix_suggestion.as_deref(),
            ),
        }
    }

    fn fix_suggestion(&self) -> Option<&str> {
        self.fix_suggestion.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_command_found() {
        // `echo` existe em qualquer sistema Unix
        let check = CommandCheck::new("Echo", "echo");
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;

        assert_eq!(result.name, "Echo");
        assert_eq!(result.status, super::super::Status::Pass);
    }

    #[tokio::test]
    async fn test_command_not_found() {
        let check =
            CommandCheck::new("FakeTool", "ferpdeferpferp").with_fix("apt install faketool");
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;

        assert_eq!(result.status, super::super::Status::Fail);
        assert_eq!(
            result.fix_suggestion,
            Some("apt install faketool".to_string())
        );
    }
}
