// Modulo output: formatacao e display dos resultados no terminal

use crate::checks::{CheckResult, Status};

// Cores ANSI para output colorido
pub const GREEN: &str = "\x1b[32m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const CYAN: &str = "\x1b[36m";
pub const DIMMED: &str = "\x1b[2m";
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";

/// Aplica cor a uma string
pub fn color(text: &str, color_code: &str) -> String {
    format!("{}{}{}", color_code, text, RESET)
}

/// Exibe os resultados dos checks no terminal
pub fn print_results(results: &[CheckResult]) {
    if results.is_empty() {
        println!("\n  No checks configured.\n");
        return;
    }

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

/// Exibe resumo final dos checks
pub fn print_summary(pass: usize, fail: usize, warn: usize) {
    let total = pass + fail + warn;
    println!(
        "  {} {} total  —  {} passed, {} failed, {} warning(s)\n",
        color("Summary:", BOLD),
        total,
        color(&pass.to_string(), GREEN),
        if fail > 0 {
            color(&fail.to_string(), RED)
        } else {
            color("0", GREEN)
        },
        if warn > 0 {
            color(&warn.to_string(), YELLOW)
        } else {
            color("0", GREEN)
        },
    );

    if fail > 0 {
        println!(
            "  {} Run `{}` to auto-resolve what's possible.\n",
            color("💡", YELLOW),
            color("checkup fix", CYAN)
        );
    }
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

    let (pass, fail, warn) = (
        results.iter().filter(|r| r.status == Status::Pass).count(),
        results.iter().filter(|r| r.status == Status::Fail).count(),
        results
            .iter()
            .filter(|r| r.status == Status::Warning)
            .count(),
    );

    let combined = serde_json::json!({
        "summary": {
            "total": results.len(),
            "pass": pass,
            "fail": fail,
            "warn": warn,
        },
        "checks": json_results,
        "fixes": fix_json,
    });

    serde_json::to_string_pretty(&combined).unwrap_or_else(|_| "{}".to_string())
}
