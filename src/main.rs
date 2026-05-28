// checkup - Environment diagnostics and auto-fix for development machines
// Entry point: parse CLI args e direciona para init, check ou fix

use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod checks;
mod config;
mod output;
mod runner;

/// CLI arguments do checkup
#[derive(Parser)]
#[command(
    name = "checkup",
    version = "0.1.0",
    about = "Environment diagnostics and auto-fix"
)]
struct Cli {
    /// Caminho para o arquivo de configuracao
    #[arg(short, long, default_value = ".checkup.toml", global = true)]
    config: PathBuf,

    /// Suprime output de checks que passaram
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Output em formato JSON
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Executa todos os checks e mostra o diagnostico
    Check {
        /// Tenta auto-corrigir problemas encontrados
        #[arg(short, long)]
        fix: bool,
    },

    /// Cria um arquivo .checkup.toml com configuracao padrao
    Init,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Init) => cmd_init(),
        Some(Commands::Check { fix }) => {
            cmd_check(&cli.config, cli.quiet, cli.json, *fix);
        }
        None => {
            // Sem subcomando: executa check por padrao
            cmd_check(&cli.config, cli.quiet, cli.json, false);
        }
    }
}

/// Comando init: cria .checkup.toml padrao
fn cmd_init() {
    let path = PathBuf::from(".checkup.toml");

    if path.exists() {
        output::display_error(".checkup.toml already exists. Remove it first to recreate.");
        std::process::exit(1);
    }

    let default_content = r#"# checkup configuration
# Defina abaixo as verificacoes para seu ambiente de desenvolvimento

# [commands.node]
# command = "node"
# fix = "fnm install 20"

# [versions.node]
# version_flag = "--version"
# expected = ">=20"
# fix = "fnm install 20"

# [postgres]
# port = 5432

# [redis]
# port = 6379

# [ports]
# free = [3000, 8080]

# [env]
# required = ["DATABASE_URL", "JWT_SECRET"]

# [envfile]
# path = ".env"
# required = ["DATABASE_URL", "JWT_SECRET"]
"#;

    match std::fs::write(&path, default_content) {
        Ok(_) => {
            println!("  {} Created {}", "✓".green().bold(), path.display());
            println!(
                "  {} Edit the file and run 'checkup' to diagnose",
                "→".cyan()
            );
        }
        Err(e) => {
            output::display_error(&format!("could not create .checkup.toml: {}", e));
            std::process::exit(1);
        }
    }
}

/// Comando check: carrega config, executa checks, mostra resultado
fn cmd_check(config_path: &PathBuf, quiet: bool, json: bool, fix: bool) {
    // Carrega configuracao
    let config = match config::CheckupConfig::load(config_path) {
        Ok(c) => c,
        Err(e) => {
            output::display_error(&e.to_string());
            eprintln!();
            println!(
                "  {} Run 'checkup init' to create a config file",
                "→".cyan()
            );
            std::process::exit(1);
        }
    };

    let project_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Executa checks em bloco async
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let results = runtime.block_on(runner::run_all(&config, &project_path));

    // Mostra resultado
    output::display_header();

    if json {
        output::display_json(&results);
    } else {
        output::display(&results, quiet);

        if fix {
            println!();
            println!("  {} Auto-fix mode", "🔧".bold());
            for result in &results {
                if let Some(suggestion) = &result.fix_suggestion {
                    println!("    {} {}: {}", "→".cyan(), result.name, suggestion);
                }
            }
        }
    }

    // Exit code: 1 se houve falhas, 0 se tudo OK
    let has_failures = results.iter().any(|r| r.status == checks::Status::Fail);
    if has_failures {
        std::process::exit(1);
    }
}
