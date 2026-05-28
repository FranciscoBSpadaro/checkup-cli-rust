// Check de servico: verifica se servico esta rodando (TCP port check)
// Conecta async na porta via tokio::net::TcpStream

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
}

#[async_trait::async_trait]
impl Check for ServiceCheck {
    fn name(&self) -> &str {
        &self.name
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
        // Porta 0 nunca deveria ter um servico
        let config = crate::config::ServiceConfig {
            port: 0,
            host: "localhost".to_string(),
        };
        let check = ServiceCheck::new("DummyService", &config);
        let ctx = Context::new(std::env::current_dir().unwrap());

        let result = check.run(&ctx).await;
        assert_eq!(result.status, super::super::Status::Fail);
    }
}
