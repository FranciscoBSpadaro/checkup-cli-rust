mod checks;
mod config;
mod output;
mod runner;

use checks::Context;
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::PathBuf;

const CONFIG_FILE: &str = "checkup.toml";

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
    #[arg(short, long, default_value = CONFIG_FILE)]
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
    /// Inicializa checkup.toml com configuracao detectada automaticamente
    Init,
}

#[derive(Clone, clap::ValueEnum)]
enum Shell {
    Bash,
    Zsh,
    Fish,
    Powershell,
    Elvish,
}

/// Extrai a versão atual de uma ferramenta executando `<cmd> <flag>`
fn detect_version(cmd: &str, flag: &str) -> Option<String> {
    use std::process::Command;
    let output = Command::new(cmd).arg(flag).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let re = regex::Regex::new(r"(\d+\.\d+(?:\.\d+)?)").ok()?;
    re.captures(&stdout)
        .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
}

/// Detecta serviços definidos em docker-compose.yml
fn detect_compose_services(project_path: &std::path::Path) -> Vec<(String, u16)> {
    let compose_path = project_path.join("docker-compose.yml");
    let content = std::fs::read_to_string(&compose_path).unwrap_or_default();
    let common_ports: &[(&str, u16)] = &[
        ("postgres", 5432),
        ("pg", 5432),
        ("postgresql", 5432),
        ("redis", 6379),
        ("mongo", 27017),
        ("mongodb", 27017),
        ("mysql", 3306),
        ("rabbitmq", 5672),
        ("kafka", 9092),
        ("elasticsearch", 9200),
    ];
    let mut services = Vec::new();
    let content_lower = content.to_lowercase();
    for (name, port) in common_ports {
        let already = services.iter().any(|(n, _)| n == name);
        if !already
            && (content_lower.contains(&format!("{name}:"))
                || content_lower.contains(&format!("image: {name}"))
                || content_lower.contains(&format!("image: {name}/")))
        {
            services.push((name.to_string(), *port));
        }
    }
    services
}

/// Detecta ferramentas instaladas no sistema para gerar config inicial
fn init_config(project_path: &std::path::Path) -> config::CheckupConfig {
    use config::{CheckupConfig, CommandConfig};
    use std::collections::HashMap;

    let mut commands = HashMap::new();
    let mut versions = HashMap::new();

    // Ferramentas: (nome, comando, fix, version_flag, min_version)
    let candidates: Vec<(&str, &str, &str, &str, Option<&str>)> = vec![
        ("node", "node", "fnm install 20", "--version", Some(">=20")),
        ("npm", "npm", "npm install -g npm", "--version", None),
        ("yarn", "yarn", "npm install -g yarn", "--version", None),
        ("pnpm", "pnpm", "npm install -g pnpm", "--version", None),
        (
            "docker",
            "docker",
            "https://docs.docker.com/get-docker/",
            "--version",
            None,
        ),
        (
            "git",
            "git",
            "https://git-scm.com/downloads",
            "--version",
            None,
        ),
        (
            "cargo",
            "cargo",
            "https://rustup.rs",
            "--version",
            Some(">=1.75"),
        ),
        (
            "python3",
            "python3",
            "https://www.python.org/downloads/",
            "--version",
            None,
        ),
        ("go", "go", "https://go.dev/dl/", "version", None),
        (
            "ruby",
            "ruby",
            "https://www.ruby-lang.org/en/documentation/installation/",
            "--version",
            None,
        ),
        ("java", "java", "https://adoptium.net/", "--version", None),
        (
            "redis-cli",
            "redis-cli",
            "https://redis.io/docs/getting-started/installation/",
            "--version",
            None,
        ),
        (
            "psql",
            "psql",
            "https://www.postgresql.org/download/",
            "--version",
            None,
        ),
        (
            "mongosh",
            "mongosh",
            "https://www.mongodb.com/docs/mongodb-shell/install/",
            "--version",
            None,
        ),
    ];

    for (name, cmd, fix, ver_flag, min_ver) in candidates {
        if which::which(cmd).is_ok() {
            commands.insert(
                name.to_string(),
                CommandConfig {
                    command: cmd.to_string(),
                    fix: Some(fix.to_string()),
                },
            );
            if let Some(min) = min_ver {
                let expected = if let Some(current) = detect_version(cmd, ver_flag) {
                    let parts: Vec<_> = current.split('.').take(2).collect();
                    format!(">={}", parts.join("."))
                } else {
                    min.to_string()
                };
                versions.insert(
                    name.to_string(),
                    config::VersionConfig {
                        version_flag: ver_flag.to_string(),
                        expected,
                        fix: Some(fix.to_string()),
                    },
                );
            }
        }
    }

    // Detecta serviços do docker-compose.yml
    let mut services = HashMap::new();
    for (name, port) in detect_compose_services(project_path) {
        services.insert(
            name.to_string(),
            config::ServiceConfig {
                host: "127.0.0.1".to_string(),
                port,
            },
        );
    }

    // Portas livres comuns
    let mut free_ports: Vec<u16> = vec![3000, 8080];

    // Tenta extrair portas do package.json
    if let Ok(content) = std::fs::read_to_string(project_path.join("package.json")) {
        for port in [3001, 4200, 5173, 5000, 8000] {
            if content.contains(&port.to_string()) && !free_ports.contains(&port) {
                free_ports.push(port);
            }
        }
    }
    free_ports.sort();

    let mut config = CheckupConfig {
        commands,
        versions,
        services,
        ports: config::PortsConfig { free: free_ports },
        ..Default::default()
    };

    // Só inclui envfile se encontrar .env ou .env.example
    let env_path = project_path.join(".env");
    let env_example = project_path.join(".env.example");
    if env_path.exists() || env_example.exists() {
        config.envfile.path = ".env".to_string();
        let example_path = if env_example.exists() {
            env_example
        } else {
            env_path
        };
        let mut keys = Vec::new();
        if let Ok(content) = std::fs::read_to_string(&example_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, _)) = line.split_once('=') {
                    let key = key.trim().to_string();
                    if !key.is_empty() {
                        keys.push(key);
                    }
                }
            }
        }
        if !keys.is_empty() {
            config.envfile.required = keys;
        }
    }

    config
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Handle `checkup init` separately — doesn't need existing config
    if let Some(Commands::Init) = &cli.command {
        let project_path =
            std::fs::canonicalize(&cli.project).unwrap_or_else(|_| cli.project.clone());
        let config_path = project_path.join(CONFIG_FILE);

        if config_path.exists() {
            eprintln!(
                "{} already exists. Remove it first if you want to reinitialize.",
                CONFIG_FILE
            );
            std::process::exit(1);
        }

        let config = init_config(&project_path);
        let toml_str = toml::to_string_pretty(&config).expect("failed to serialize config");

        // Write with header comment
        let content = format!(
            "# checkup.toml — Environment diagnostics config\n\
             # Generated by `checkup init` on {}\n\
             # Edit this file to add/remove checks for your project.\n\n{}\n",
            chrono::Local::now().format("%Y-%m-%d"),
            toml_str
        );

        if let Err(e) = std::fs::write(&config_path, &content) {
            eprintln!("Error writing {}: {}", CONFIG_FILE, e);
            std::process::exit(1);
        }

        let cmd_count = config.commands.len();
        println!(
            "  {} Created {} with {} detected tool(s).",
            output::color("✓", output::GREEN),
            CONFIG_FILE,
            cmd_count
        );
        println!("  Edit the file to add version checks, services, ports, and more.");
        println!("  Then run `checkup` to see your environment status.");
        return;
    }

    // Carrega configuracao
    let config = match config::load_config(&cli.config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading config: {}", e);
            eprintln!("Run `checkup init` to create a config file.");
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
            let results = runner::run_all(&checks, &ctx).await;
            let fix_results = runner::fix_all(&checks, &results, &ctx).await;

            if cli.json {
                println!("{}", output::format_json(&results, &fix_results));
            } else if !cli.quiet {
                output::print_results(&results);
                if !fix_results.is_empty() {
                    println!("{}", output::color("\nAuto-fix results:", output::CYAN));
                    output::print_results(&fix_results);
                }
            }

            let (pass, fail, warn) = runner::summarize(&results);
            if !cli.quiet {
                output::print_summary(pass, fail, warn);
            }

            let has_failures = results.iter().any(|r| r.status == checks::Status::Fail);
            std::process::exit(if has_failures { 1 } else { 0 });
        }
        Some(Commands::List) => {
            if checks.is_empty() {
                println!(
                    "  No checks configured. Add sections to your {}",
                    CONFIG_FILE
                );
            } else {
                for check in &checks {
                    println!("  {}", check.name());
                }
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

            let (pass, fail, warn) = runner::summarize(&results);
            if !cli.quiet {
                output::print_summary(pass, fail, warn);
            }

            std::process::exit(if fail > 0 { 1 } else { 0 });
        }
        Some(Commands::Init) => unreachable!(),
    }
}
