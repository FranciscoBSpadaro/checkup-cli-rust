// Check de porta: verifica se uma porta esta livre para uso
// Tenta abrir um TcpListener na porta — se conseguir, esta livre
// Suporta auto-fix: identifica o processo e oferece mata-lo

use super::{Check, CheckResult, Context};
use std::process::Command;

// ── Platform-specific helpers ──────────────────────────────────────────────

/// Build a shell command that runs `cmd` and returns its stdout.
/// On Unix we use `sh -c`; on Windows we use `cmd /C`.
fn shell_cmd(cmd: &str) -> Command {
    let mut c = Command::new(if cfg!(windows) { "cmd" } else { "sh" });
    if cfg!(windows) {
        c.arg("/C");
    } else {
        c.arg("-c");
    }
    c.arg(cmd);
    c
}

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
        #[cfg(not(windows))]
        {
            // Try lsof (Linux/macOS)
            if let Ok(output) =
                shell_cmd(&format!("lsof -i :{port} -t 2>/dev/null | head -1")).output()
            {
                let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !pid.is_empty() {
                    return Some(pid);
                }
            }

            // Try ss + grep (Linux)
            if let Ok(output) = shell_cmd(&format!(
                "ss -tlnp 'sport = :{port}' 2>/dev/null | sed -n 's/.*pid=\\([0-9]*\\).*/\\1/p' | head -1"
            )).output() {
                let pid = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !pid.is_empty() {
                    return Some(pid);
                }
            }
        }

        #[cfg(windows)]
        {
            // Windows: use netstat -ano to find the PID
            if let Ok(output) = Command::new("netstat").args(["-ano"]).output() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    // Look for lines like: TCP    0.0.0.0:8080    ...    LISTENING    1234
                    if line.contains(&format!(":{port}")) && line.contains("LISTENING") {
                        if let Some(pid) = line.split_whitespace().last() {
                            return Some(pid.to_string());
                        }
                    }
                }
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
        #[cfg(not(windows))]
        return Some("kill the process using the port");
        #[cfg(windows)]
        return Some("taskkill /PID <pid> (find pid with netstat -ano)");
    }

    async fn fix(&self, _ctx: &Context) -> Option<CheckResult> {
        let pid = Self::find_process_on_port(self.port)?;

        // Try graceful kill first
        #[cfg(not(windows))]
        let kill_result = tokio::process::Command::new("kill")
            .arg(&pid)
            .status()
            .await
            .ok()?;

        #[cfg(windows)]
        let kill_result = tokio::process::Command::new("taskkill")
            .args(["/PID", &pid])
            .status()
            .await
            .ok()?;

        if kill_result.success() {
            // Wait for the process to die
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

            // Check if the port is now free
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
                    // Force kill
                    #[cfg(not(windows))]
                    let _ = tokio::process::Command::new("kill")
                        .arg("-9")
                        .arg(&pid)
                        .status()
                        .await;

                    #[cfg(windows)]
                    let _ = tokio::process::Command::new("taskkill")
                        .args(["/F", "/PID", &pid])
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
                        Err(e) => {
                            #[cfg(not(windows))]
                            let hint = "check manually with lsof or ss";
                            #[cfg(windows)]
                            let hint = "check manually with netstat -ano";
                            Some(CheckResult::fail(
                                &format!("Port {}", self.port),
                                &format!(
                                    "port {} still in use after force kill on pid {}: {}",
                                    self.port, pid, e
                                ),
                                Some(hint),
                            ))
                        }
                    }
                }
            }
        } else {
            #[cfg(not(windows))]
            let hint = format!("sudo kill -9 {pid}");
            #[cfg(windows)]
            let hint = format!("taskkill /F /PID {pid}");
            Some(CheckResult::fail(
                &format!("Port {}", self.port),
                &format!(
                    "auto-fix failed: could not kill pid {} on port {}",
                    pid, self.port
                ),
                Some(&hint),
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
