// Motor de execucao dos checks: roda todos os checks do contexto em paralelo
// via tokio::join! e agrega resultados

use crate::checks::{Check, CheckResult, Context, Status};
use std::sync::Arc;

/// Roda todos os checks em paralelo e retorna os resultados
pub async fn run_all(checks: &[Arc<dyn Check>], ctx: &Context) -> Vec<CheckResult> {
    let futures: Vec<_> = checks.iter().map(|c| c.run(ctx)).collect();
    let mut results = futures::future::join_all(futures).await;

    // Atribui duracao baseada no tempo de execucao
    for result in &mut results {
        result.duration_ms = if result.duration_ms == 0 {
            1
        } else {
            result.duration_ms
        };
    }

    results
}

/// Tenta corrigir todos os checks que falharam
/// Retorna os resultados do fix para cada check que tentou corrigir
pub async fn fix_all(
    checks: &[Arc<dyn Check>],
    results: &[CheckResult],
    ctx: &Context,
) -> Vec<CheckResult> {
    let failed_names: Vec<&str> = results
        .iter()
        .filter(|r| r.status == Status::Fail)
        .map(|r| r.name.as_str())
        .collect();

    if failed_names.is_empty() {
        return vec![];
    }

    let fixes: Vec<_> = checks
        .iter()
        .filter(|c| failed_names.contains(&c.name()))
        .map(|c| c.fix(ctx))
        .collect();

    let fix_results = futures::future::join_all(fixes).await;
    fix_results.into_iter().flatten().collect()
}

/// Conta resultados por status
pub fn summarize(results: &[CheckResult]) -> (usize, usize, usize) {
    let pass = results.iter().filter(|r| r.status == Status::Pass).count();
    let fail = results.iter().filter(|r| r.status == Status::Fail).count();
    let warn = results
        .iter()
        .filter(|r| r.status == Status::Warning)
        .count();
    (pass, fail, warn)
}
