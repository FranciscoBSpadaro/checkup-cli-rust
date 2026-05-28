// Modulo output: formatacao e display dos resultados no terminal
// Usa colored para output colorido e respeita flags (--json, --quiet)

use crate::checks::{CheckResult, Status};
use colored::Colorize;

/// Exibe os resultados dos checks no terminal
pub fn display(results: &[CheckResult], quiet: bool) {
    let mut passed = 0;
    let mut failed = 0;
    let mut warnings = 0;

    for result in results {
        match result.status {
            Status::Pass => {
                passed += 1;
                if !quiet {
                    println!("  {}  {}", "✓".green().bold(), result.name.green());
                    println!("    {}", result.message.dimmed());
                }
            }
            Status::Fail => {
                failed += 1;
                println!("  {}  {}", "✗".red().bold(), result.name.red());
                println!("    {}", result.message);
                if let Some(fix) = &result.fix_suggestion {
                    println!("    {} {}", "└─ Fix:".yellow(), fix.yellow());
                }
            }
            Status::Warning => {
                warnings += 1;
                println!("  {}  {}", "⚠".yellow().bold(), result.name.yellow());
                println!("    {}", result.message);
            }
            Status::Skipped => {
                if !quiet {
                    println!("  {}  {}", "○".dimmed(), result.name.dimmed());
                }
            }
        }
    }

    // Linha separadora
    println!();
    println!(
        "  {} {} passed, {} failed, {} warnings",
        "Summary:".bold(),
        passed.to_string().green(),
        failed.to_string().red(),
        warnings.to_string().yellow(),
    );

    if failed > 0 {
        println!(
            "  {} Run 'checkup --fix' to auto-resolve what's possible",
            "→".cyan()
        );
    }
}

/// Exibe os resultados em formato JSON
pub fn display_json(results: &[CheckResult]) {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "status": format!("{:?}", r.status),
                "message": r.message,
                "fix": r.fix_suggestion,
            })
        })
        .collect();

    match serde_json::to_string_pretty(&json_results) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Error serializing JSON: {}", e),
    }
}

/// Exibe uma mensagem de erro formatada
pub fn display_error(msg: &str) {
    eprintln!("  {} {}", "Error:".red().bold(), msg);
}

/// Exibe uma mensagem de boas-vindas
pub fn display_header() {
    println!();
    println!(
        "  {} {}",
        "checkup".cyan().bold(),
        "v0.1.0 — Environment Diagnostics".dimmed()
    );
    println!();
}
