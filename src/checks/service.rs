// Check de servico: verifica se servico esta rodando (TCP port check)
// Conecta async na porta via tokio::net::TcpStream
// Suporte a auto-fix via docker compose quando detectado

use super::{Check, CheckResult, Context};
use crate::config::ServiceConfig;

/// Check que verifica se um servico esta respondendo em uma porta TCP
pub struct ServiceCheck {
    /// Nome legivel (ex: "PostgreSQL")
    name: String,
    /// Host para verificar
    host: String,
    /// Porta TCP
    port: u16,
}

impl ServiceCheck {
    /// Cria um novo check de servico a partir da configuracao
    pub fn new(name: &str, config: &ServiceConfig) -> Self {
        Self {
            name: name.to_string(),
            host: config.host.clone(),
            port: config.port,
        }
    }

    /// Tenta detectar o gerenciador de containers e subir o servico
    async fn try_fix(check: &ServiceCheck) -> Option<CheckResult> {
        if which::which("docker").is_ok() {
            let project_dir = std::env::current_dir().ok()?;

            for compose_name in &[
                "docker-compose.yml",
                "docker-compose.yaml",
                "compose.yml",
                "compose.yaml",
            ] {
                if project_dir.join(compose_name).exists() {
                    let status = tokio::process::Command::new("docker")
                        .args(["compose", "up", "-d"])
                        .current_dir(&project_dir)
                        .status()
                        .await
                        .ok()?;

                    if status.success() {
                        // Aguarda o servico iniciar
                        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

                        let addr = format!("{}:{}", check.host, check.port);
                        match tokio::net::TcpStream::connect(&addr).await {
                            Ok(_) => {
                                return Some(CheckResult::pass(
                                    &check.name,
                                    "service started via docker compose",
                                ));
                            }
                            Err(_) => {
                                return Some(CheckResult::fail(
                                    &check.name,
                                    "docker compose up succeeded but service not yet responding",
                                    Some("wait for service to fully start"),
                                ));
                            }
                        }
                    }
                }
            }
        }

        Some(CheckResult::fail(
            &check.name,
            "auto-fix: no docker compose found for this service",
            Some("start the service manually"),
        ))
    }
}

#[async_trait::async_trait]
impl Check for ServiceCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn fix_suggestion(&self) -> Option<&str> {
        Some("start the service (e.g., docker compose up)")
    }

    async fn fix(&self, _ctx: &Context) -> Option<CheckResult> {
        ServiceCheck::try_fix(self).await
    }

    async fn run(&self, _ctx: &Context) -> CheckResult {
        let addr = format!("{}:{}", self.host, self.port);

        match tokio::net::TcpStream::connect(&addr).await {
            Ok(_) => CheckResult::pass(&self.name, &format!("running on :{}", self.port)),
            Err(e) => CheckResult::fail(
                &self.name,
                &format!("not responding on :{} ({})", self.port, e),
                Some("start the service"),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_not_running() {
        // Porta alta que devera estar livre
        let config = ServiceConfig {
            port: 1,
            host: "127.0.0.1".to_string(),
        };
        let check = ServiceCheck::new("DummyService", &config);
        let ctx = Context::new(std::env::current_dir().unwrap());

        let result = check.run(&ctx).await;
        assert_eq!(result.status, super::super::Status::Fail);
    }
}
