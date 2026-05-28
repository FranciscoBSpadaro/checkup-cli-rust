// Check de versao: verifica versao de um programa (semver)
// Parseia output de --version e compara com versao esperada

use super::{Check, CheckResult, Context};
use crate::config::VersionConfig;

/// Check que verifica a versao de uma ferramenta
pub struct VersionCheck {
    /// Nome legivel (ex: "Node.js")
    name: String,
    /// Caminho do executavel
    command: String,
    /// Flag para obter versao (ex: "--version")
    version_flag: String,
    /// Versao esperada como constraint semver (ex: ">=20")
    expected: String,
    /// Sugestao de correcao
    fix_suggestion: Option<String>,
}

impl VersionCheck {
    /// Cria um novo check de versao a partir da configuracao
    pub fn new(name: &str, command: &str, config: &VersionConfig) -> Self {
        Self {
            name: name.to_string(),
            command: command.to_string(),
            version_flag: config.version_flag.clone(),
            expected: config.expected.clone(),
            fix_suggestion: config.fix.clone(),
        }
    }

    /// Extrai o numero de versao de uma string tipo "v20.10.0" ou "20.10.0"
    fn extract_version(output: &str) -> Option<String> {
        // Procura por padrao de versao (X.Y.Z ou X.Y) na saída
        for word in output.split_whitespace() {
            let cleaned = word.trim_matches(|c: char| !c.is_ascii_digit() && c != '.');
            if cleaned.split('.').count() >= 2 {
                return Some(cleaned.to_string());
            }
        }
        None
    }

    /// Compara versoes no formato simplificado (apenas major, ignora minor/patch)
    /// Retorna true se a versao atende ao constraint (ex: ">=20")
    fn version_meets_constraint(version: &str, constraint: &str) -> bool {
        let major: u32 = version
            .split('.')
            .next()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        if let Some(rest) = constraint.strip_prefix(">=") {
            let expected: u32 = rest.trim().parse().unwrap_or(0);
            major >= expected
        } else if let Some(rest) = constraint.strip_prefix('>') {
            let expected: u32 = rest.trim().parse().unwrap_or(0);
            major > expected
        } else if let Some(rest) = constraint.strip_prefix("<=") {
            let expected: u32 = rest.trim().parse().unwrap_or(0);
            major <= expected
        } else if let Some(rest) = constraint.strip_prefix('<') {
            let expected: u32 = rest.trim().parse().unwrap_or(0);
            major < expected
        } else if let Some(rest) = constraint.strip_prefix("==") {
            let expected: u32 = rest.trim().parse().unwrap_or(0);
            major == expected
        } else {
            // Sem constraint reconhecido, compara igualdade
            let expected: u32 = constraint.trim().parse().unwrap_or(0);
            major == expected
        }
    }
}

impl VersionCheck {
    /// Detecta gerenciador de versao disponivel no PATH
    fn detect_version_manager() -> Option<String> {
        for manager in &["fnm", "mise", "asdf", "nvm"] {
            if which::which(manager).is_ok() {
                return Some(manager.to_string());
            }
        }
        None
    }
}

#[async_trait::async_trait]
impl Check for VersionCheck {
    fn name(&self) -> &str {
        &self.name
    }

    fn fix_suggestion(&self) -> Option<&str> {
        self.fix_suggestion.as_deref()
    }

    async fn fix(&self, _ctx: &Context) -> Option<CheckResult> {
        // Tenta gerenciadores de versao: fnm, nvm, asdf, mise
        if let Some(manager) = VersionCheck::detect_version_manager() {
            // Extrai major version da constraint (ex: ">=20" -> "20")
            let version = self
                .expected
                .trim_start_matches(|c: char| !c.is_ascii_digit());
            if !version.is_empty() {
                let use_cmd = format!("{} use {}", manager, version);
                let status = tokio::process::Command::new("sh")
                    .arg("-c")
                    .arg(&use_cmd)
                    .status()
                    .await
                    .ok()?;

                if status.success() {
                    return Some(CheckResult::pass(
                        &self.name,
                        &format!("switched to version {} via {}", version, manager),
                    ));
                }
            }
        }

        // Fallback: usa a sugestao de fix do config
        if let Some(suggestion) = &self.fix_suggestion {
            let status = tokio::process::Command::new("sh")
                .arg("-c")
                .arg(suggestion)
                .status()
                .await
                .ok()?;

            if status.success() {
                return Some(CheckResult::pass(
                    &self.name,
                    &format!("auto-fixed: ran '{}'", suggestion),
                ));
            }
        }

        Some(CheckResult::fail(
            &self.name,
            "auto-fix: could not upgrade version automatically",
            self.fix_suggestion.as_deref(),
        ))
    }

    async fn run(&self, _ctx: &Context) -> CheckResult {
        // Executa o comando com a flag de versao
        let output = match tokio::process::Command::new(&self.command)
            .arg(&self.version_flag)
            .output()
            .await
        {
            Ok(out) => out,
            Err(e) => {
                return CheckResult::fail(
                    &self.name,
                    &format!(
                        "failed to run '{} {}': {}",
                        self.command, self.version_flag, e
                    ),
                    self.fix_suggestion.as_deref(),
                );
            }
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{} {}", stdout, String::from_utf8_lossy(&output.stderr));
        let combined = combined.trim();

        // Extrai o numero de versao da saida
        let version = match Self::extract_version(combined) {
            Some(v) => v,
            None => {
                return CheckResult::fail(
                    &self.name,
                    &format!(
                        "could not parse version from output: '{}'",
                        combined.lines().next().unwrap_or("")
                    ),
                    self.fix_suggestion.as_deref(),
                );
            }
        };

        // Compara com a versao esperada
        if Self::version_meets_constraint(&version, &self.expected) {
            CheckResult::pass(&self.name, &format!("{} (>= {})", version, self.expected))
        } else {
            CheckResult::fail(
                &self.name,
                &format!("found {}, expected {}", version, self.expected),
                self.fix_suggestion.as_deref(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_version() {
        assert_eq!(
            VersionCheck::extract_version("v20.10.0"),
            Some("20.10.0".to_string())
        );
        assert_eq!(
            VersionCheck::extract_version("rustc 1.75.0"),
            Some("1.75.0".to_string())
        );
        assert_eq!(
            VersionCheck::extract_version("Python 3.11.5"),
            Some("3.11.5".to_string())
        );
        assert_eq!(
            VersionCheck::extract_version("18.2.0"),
            Some("18.2.0".to_string())
        );
    }

    #[test]
    fn test_version_constraint_gte() {
        assert!(VersionCheck::version_meets_constraint("20.10.0", ">=20"));
        assert!(VersionCheck::version_meets_constraint("21.0.0", ">=20"));
        assert!(!VersionCheck::version_meets_constraint("18.2.0", ">=20"));
    }

    #[test]
    fn test_version_constraint_eq() {
        assert!(VersionCheck::version_meets_constraint("20.0.0", "==20"));
        assert!(!VersionCheck::version_meets_constraint("21.0.0", "==20"));
    }

    #[test]
    fn test_version_constraint_lt() {
        assert!(VersionCheck::version_meets_constraint("18.2.0", "<20"));
        assert!(!VersionCheck::version_meets_constraint("20.0.0", "<20"));
    }
}
