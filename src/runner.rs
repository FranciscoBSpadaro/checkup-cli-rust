// Modulo runner: orquestra checks em paralelo via tokio::join!
// Recebe CheckupConfig, instancia cada check, retorna Vec<CheckResult>

use crate::checks::{
    Check, CheckResult, CommandCheck, Context, EnvFileCheck, EnvVarCheck, PortCheck, ServiceCheck,
    VersionCheck,
};
use crate::config::CheckupConfig;
use std::path::Path;

/// Executa todos os checks em paralelo e retorna os resultados
pub async fn run_all(config: &CheckupConfig, project_path: &Path) -> Vec<CheckResult> {
    let ctx = Context::new(project_path.to_path_buf());
    let mut checks: Vec<Box<dyn Check>> = Vec::new();

    // Registra checks de comando
    for (name, cmd_config) in &config.commands {
        let mut check = CommandCheck::new(name, &cmd_config.command);
        if let Some(fix) = &cmd_config.fix {
            check = check.with_fix(fix);
        }
        checks.push(Box::new(check));
    }

    // Registra checks de versao
    for (name, ver_config) in &config.versions {
        // Assume que o comando e o mesmo nome (minusculas) — pode ser refinado
        let command = name.to_lowercase().replace([' ', '.'], "");
        checks.push(Box::new(VersionCheck::new(name, &command, ver_config)));
    }

    // Registra checks de servico
    for (name, svc_config) in &config.services {
        checks.push(Box::new(ServiceCheck::new(name, svc_config)));
    }

    // Registra checks de porta livre
    for port in &config.ports.free {
        checks.push(Box::new(PortCheck::new(*port)));
    }

    // Registra check de envfile (um unico check que valida todas as chaves)
    if !config.envfile.required.is_empty() {
        checks.push(Box::new(EnvFileCheck::new(&config.envfile)));
    }

    // Registra check de variaveis de ambiente
    if !config.env.required.is_empty() {
        checks.push(Box::new(EnvVarCheck::new("Environment", &config.env)));
    }

    // Executa todos os checks em paralelo
    let futures: Vec<_> = checks.iter().map(|check| check.run(&ctx)).collect();
    let results = futures::future::join_all(futures).await;

    results
}

/// Tenta auto-corrigir os checks que falharam
pub async fn fix_all(
    results: &[CheckResult],
    config: &CheckupConfig,
    project_path: &Path,
) -> Vec<(String, String)> {
    let mut fixes = Vec::new();

    for result in results {
        if let Some(suggestion) = &result.fix_suggestion {
            fixes.push((result.name.clone(), suggestion.clone()));
        }
    }

    fixes
}
