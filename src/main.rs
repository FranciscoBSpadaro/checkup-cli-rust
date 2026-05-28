mod checks;
mod config;
mod output;
mod runner;

use checks::Context;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "checkup",
    version,
    about = "Verify your development environment",
    long_about = "A fast CLI tool to verify your development environment.\n\
                  Checks commands, services, ports, env vars, and more."
)]
struct Cli {
    /// Caminho para o arquivo de configuracao
    #[arg(short, long, default_value = "checkup.toml")]
    config: PathBuf,

    /// Caminho do projeto (default: diretorio atual)
    #[arg(short, long, default_value = ".")]
    project: PathBuf,

    /// Saida em formato JSON
    #[arg(long)]
    json: bool,

    /// Modo silencioso (apenas codigo de saida)
    #[arg(short, long)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Executa todos os checks
    Check,
    /// Tenta corrigir problemas encontrados
    Fix,
    /// Lista todos os checks configurados
    List,
    /// Gera shell completions
    Completions {
        /// Shell (bash, zsh, fish, powershell, elvish)
        #[arg(value_enum)]
        shell: Shell,
    },
}

#[derive(Clone, clap::ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Carrega configuracao
    let config = match config::load_config(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            std::process::exit(1);
        }
    };

    // Resolve caminho do projeto
    let project_path = std::fs::canonicalize(&cli.project).unwrap_or_else(|_| cli.project.clone());

    // Carrega .env file se existir
    let mut env_overrides = HashMap::new();
    let env_file_path = project_path.join(".env");
    if env_file_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_file_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, value)) = line.split_once('=') {
                    env_overrides.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    let ctx = Context::with_env(project_path.clone(), env_overrides);

    // Monta lista de checks
    let checks = config::build_checks(&config, &project_path);

    match cli.command {
        Some(Commands::Fix) => {
            // Primeiro roda os checks
            let results = runner::run_all(&checks, &ctx).await;

            // Depois tenta fix nos que falharam
            let fix_results = runner::fix_all(&checks, &results, &ctx).await;

            if cli.json {
                println!("{}", output::format_json(&results, &fix_results));
            } else if !cli.quiet {
                output::print_results(&results);
                if !fix_results.is_empty() {
                    println!("\n{}", output::color("Auto-fix results:", output::CYAN));
                    output::print_results(&fix_results);
                }
            }

            let has_failures = results.iter().any(|r| r.status == checks::Status::Fail);
            std::process::exit(if has_failures { 1 } else { 0 });
        }
        Some(Commands::List) => {
            for check in &checks {
                println!("  {}", check.name());
            }
        }
        Some(Commands::Completions { shell }) => {
            let shell = match shell {
                Shell::Bash => clap_complete::Shell::Bash,
                Shell::Zsh => clap_complete::Shell::Zsh,
                Shell::Fish => clap_complete::Shell::Fish,
                Shell::Powershell => clap_complete::Shell::PowerShell,
                Shell::Elvish => clap_complete::Shell::Elvish,
            };
            clap_complete::generate(
                shell,
                &mut <Cli as clap::CommandFactory>::command(),
                "checkup",
                &mut std::io::stdout(),
            );
        }
        Some(Commands::Check) | None => {
            let results = runner::run_all(&checks, &ctx).await;

            if cli.json {
                println!("{}", output::format_json(&results, &[]));
            } else if !cli.quiet {
                output::print_results(&results);
            }

            let (pass, fail, _warn) = runner::summarize(&results);
            if !cli.quiet {
                println!(
                    "\n{} passed, {} failed",
                    output::color(&pass.to_string(), output::GREEN),
                    output::color(&fail.to_string(), output::RED),
                );
            }

            std::process::exit(if fail > 0 { 1 } else { 0 });
        }
    }
}
