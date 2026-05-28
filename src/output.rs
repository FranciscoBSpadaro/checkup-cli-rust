// Modulo output: formatacao e display dos resultados no terminal

use crate::checks::{CheckResult, Status};

// Cores ANSI para output colorido
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const DIMMED: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";

/// Aplica cor a uma string
pub fn color(text: &str, color_code: &str) -> String {
    format!("{}{}{}", color_code, text, RESET)
}

/// Exibe os resultados dos checks no terminal
pub fn print_results(results: &[CheckResult]) {
    println!();
    for result in results {
        match result.status {
            Status::Pass => {
                println!("  {}  {}", color("✓", GREEN), color(&result.name, GREEN));
                println!("    {}", color(&result.message, DIMMED));
            }
            Status::Fail => {
                println!("  {}  {}", color("✗", RED), color(&result.name, RED));
                println!("    {}", result.message);
                if let Some(fix) = &result.suggestion {
                    println!("    {} {}", color("└─ Fix:", YELLOW), color(fix, YELLOW));
                }
            }
            Status::Warning => {
                println!("  {}  {}", color("⚠", YELLOW), color(&result.name, YELLOW));
                println!("    {}", result.message);
            }
        }
    }
    println!();
}

/// Exibe os resultados em formato JSON
pub fn format_json(results: &[CheckResult], fix_results: &[CheckResult]) -> String {
    let json_results: Vec<serde_json::Value> = results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "status": format!("{:?}", r.status),
                "message": r.message,
                "suggestion": r.suggestion,
                "duration_ms": r.duration_ms,
            })
        })
        .collect();

    let fix_json: Vec<serde_json::Value> = fix_results
        .iter()
        .map(|r| {
            serde_json::json!({
                "name": r.name,
                "status": format!("{:?}", r.status),
                "message": r.message,
                "suggestion": r.suggestion,
                "duration_ms": r.duration_ms,
            })
        })
        .collect();

    let combined = serde_json::json!({
        "checks": json_results,
        "fixes": fix_json,
    });

    serde_json::to_string_pretty(&combined).unwrap_or_else(|_| "{}".to_string())
}
