// Check de porta: verifica se uma porta esta livre para uso
// Tenta abrir um TcpListener na porta — se conseguir, esta livre
// Suporta auto-fix: identifica o processo e oferece mata-lo

use super::{Check, CheckResult, Context};
use std::process::Command;

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

    /// Tenta encontrar o PID do processo usando a porta
    fn find_process_on_port(port: u16) -> Option<String> {
        // Tenta lsof (Linux/macOS)
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!("lsof -i :{} -t 2>/dev/null | head -1", port))
            .output()
        {
            let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pid.is_empty() {
                return Some(pid);
            }
        }

        // Tenta ss + grep (Linux)
        if let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg(format!(
                "ss -tlnp 'sport = :{}' 2>/dev/null | sed -n 's/.*pid=\\([0-9]*\\).*/\\1/p' | head -1",
                port
            ))
            .output()
        {
            let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pid.is_empty() {
                return Some(pid);
            }
        }

        None
    }
}

#[async_trait::async_trait]
impl Check for PortCheck {
    fn name(&self) -> &str {
        "Port"
    }

    fn fix_suggestion(&self) -> Option<&str> {
        Some("kill the process using the port")
    }

    async fn fix(&self, _ctx: &Context) -> Option<CheckResult> {
        let pid = Self::find_process_on_port(self.port)?;

        // Tenta kill gracioso primeiro
        let kill_result = tokio::process::Command::new("kill")
            .arg(&pid)
            .status()
            .await
            .ok()?;

        if kill_result.success() {
            // Aguarda o processo morrer
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Verifica se a porta ficou livre
            let addr = format!("127.0.0.1:{}", self.port);
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => {
                    drop(listener);
                    Some(CheckResult::pass(
                        &format!("Port {}", self.port),
                        &format!("port {} freed (killed pid {})", self.port, pid),
                    ))
                }
                Err(_) => {
                    // Forca kill
                    let _ = tokio::process::Command::new("kill")
                        .arg("-9")
                        .arg(&pid)
                        .status()
                        .await;

                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    match tokio::net::TcpListener::bind(&addr).await {
                        Ok(listener) => {
                            drop(listener);
                            Some(CheckResult::pass(
                                &format!("Port {}", self.port),
                                &format!("port {} freed (force killed pid {})", self.port, pid),
                            ))
                        }
                        Err(e) => Some(CheckResult::fail(
                            &format!("Port {}", self.port),
                            &format!(
                                "port {} still in use after kill -9 on pid {}: {}",
                                self.port, pid, e
                            ),
                            Some("check manually with lsof or ss"),
                        )),
                    }
                }
            }
        } else {
            Some(CheckResult::fail(
                &format!("Port {}", self.port),
                &format!(
                    "auto-fix failed: could not kill pid {} on port {}",
                    pid, self.port
                ),
                Some(&format!("sudo kill -9 {}", pid)),
            ))
        }
    }

    async fn run(&self, _ctx: &Context) -> CheckResult {
        let addr = format!("127.0.0.1:{}", self.port);

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

    #[tokio::test]
    async fn test_high_port_is_free() {
        // Portas altas (>49152) raramente estao em uso
        let check = PortCheck::new(65_123);
        let ctx = Context::new(std::env::current_dir().unwrap());
        let result = check.run(&ctx).await;
        assert_eq!(result.status, crate::checks::Status::Pass);
    }
}
