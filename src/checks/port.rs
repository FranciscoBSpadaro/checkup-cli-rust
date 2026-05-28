// Check de porta: verifica se uma porta esta livre para uso
// Tenta abrir um TcpListener na porta — se conseguir, esta livre

use super::{Check, CheckResult, Context};

/// Check que verifica se uma porta TCP esta livre
pub struct PortCheck {
    /// Porta para verificar
    port: u16,
}

impl PortCheck {
    /// Cria um novo check de porta
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

#[async_trait::async_trait]
impl Check for PortCheck {
    fn name(&self) -> &str {
        "Port"
    }

    async fn run(&self, _ctx: &Context) -> CheckResult {
        let addr = format!("0.0.0.0:{}", self.port);

        match tokio::net::TcpListener::bind(&addr).await {
            Ok(listener) => {
                // Precisa manter o listener vivo ate o fim do check
                drop(listener);
                CheckResult::pass(
                    &format!("Port {}", self.port),
                    &format!("port {} is free", self.port),
                )
            }
            Err(e) => CheckResult::fail(
                &format!("Port {}", self.port),
                &format!("port {} is already in use ({})", self.port, e),
                Some(&format!("kill the process using port {}", self.port)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_port_check_creation() {
        let check = PortCheck::new(3000);
        assert_eq!(check.name(), "Port");
    }
}
