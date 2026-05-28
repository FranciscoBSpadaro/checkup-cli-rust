// Testes de integracao do checkup
// Usa tempdir para criar projetos simulados e testar o ciclo completo

use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Helper: cria um diretorio temporario com arquivos
fn setup_temp_project() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let project_path = dir.path();

    // Cria checkup.toml minimo
    let config = r#"
[commands.git]
command = "git"

[rust]
version = ">=1.70"
"#;
    fs::write(project_path.join("checkup.toml"), config).unwrap();

    dir
}

/// Helper: cria um contexto de teste
fn test_context(dir: &Path) -> checkup::checks::Context {
    let mut env = HashMap::new();
    env.insert("HOME".to_string(), "/home/test".to_string());
    checkup::checks::Context::with_env(dir.to_path_buf(), env)
}

#[tokio::test]
async fn test_full_check_cycle_all_passing() {
    let _project = setup_temp_project();
    let config = checkup::config::load_config(&_project.path().join("checkup.toml")).unwrap();
    let ctx = test_context(_project.path());
    let checks = checkup::config::build_checks(&config, _project.path());

    let results = checkup::runner::run_all(&checks, &ctx).await;

    // Git deve passar (esta instalado)
    assert!(!results.is_empty(), "should have at least one check result");

    // Verifica que Git passou
    let git_result = results.iter().find(|r| r.name == "git");
    if let Some(git) = git_result {
        assert_eq!(git.status, checkup::checks::Status::Pass);
    }
}

#[tokio::test]
async fn test_full_check_cycle_with_missing_command() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[commands.fakecommand123]
command = "fakecommand123"
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let cmd_result = results.iter().find(|r| r.name == "fakecommand123");
    assert!(cmd_result.is_some());

    let cmd = cmd_result.unwrap();
    assert_eq!(cmd.status, checkup::checks::Status::Fail);
}

#[tokio::test]
async fn test_envfile_check_with_missing_env() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[envfile]
path = ".env.example"
required_keys = ["DATABASE_URL", "API_KEY"]
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    // Nao cria nenhum arquivo — deve falhar
    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let envfile_result = results.iter().find(|r| r.name == ".env");
    assert!(envfile_result.is_some());
    assert_eq!(
        envfile_result.unwrap().status,
        checkup::checks::Status::Fail
    );
}

#[tokio::test]
async fn test_envfile_fix_creates_from_example() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[envfile]
path = ".env"
required_keys = ["DATABASE_URL", "API_KEY"]
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    // Cria .env.example com as chaves
    fs::write(
        dir.path().join(".env.example"),
        "DATABASE_URL=postgres://localhost/mydb\nAPI_KEY=test-key\n",
    )
    .unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let envfile_result = results.iter().find(|r| r.name == ".env");
    assert!(envfile_result.is_some());
    assert_eq!(
        envfile_result.unwrap().status,
        checkup::checks::Status::Fail
    );

    // Roda o fix
    let fix_results = checkup::runner::fix_all(&checks, &results, &ctx).await;
    let fix = fix_results.iter().find(|r| r.name == ".env");
    assert!(fix.is_some());
    assert_eq!(fix.unwrap().status, checkup::checks::Status::Pass);

    // Verifica que o arquivo foi criado
    assert!(dir.path().join(".env").exists());
    let content = fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(content.contains("DATABASE_URL"));
    assert!(content.contains("API_KEY"));
}

#[tokio::test]
async fn test_envfile_fix_creates_stubs_when_no_example() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[envfile]
path = ".env"
required_keys = ["MY_VAR"]
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let fix_results = checkup::runner::fix_all(&checks, &results, &ctx).await;
    let fix = fix_results.iter().find(|r| r.name == ".env");
    assert!(fix.is_some());
    assert_eq!(fix.unwrap().status, checkup::checks::Status::Pass);

    // Verifica conteudo
    let content = fs::read_to_string(dir.path().join(".env")).unwrap();
    assert!(content.contains("MY_VAR="));
}

#[tokio::test]
async fn test_envvar_check_all_set() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[env]
required = ["HOME"]
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let env_result = results.iter().find(|r| r.name == "Environment");
    assert!(env_result.is_some());
    assert_eq!(env_result.unwrap().status, checkup::checks::Status::Pass);
}

#[tokio::test]
async fn test_envvar_check_missing() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[env]
required = ["XYZ_MISSING_VAR_12345"]
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let env_result = results.iter().find(|r| r.name == "Environment");
    assert!(env_result.is_some());
    assert_eq!(env_result.unwrap().status, checkup::checks::Status::Fail);
}

#[tokio::test]
async fn test_port_check_free_port() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[ports]
free = [65123]
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    let port_result = results.iter().find(|r| r.name.starts_with("Port"));
    assert!(port_result.is_some(), "should find Port check result");
    assert_eq!(port_result.unwrap().status, checkup::checks::Status::Pass);
}

#[tokio::test]
async fn test_run_all_and_summarize() {
    let dir = tempfile::tempdir().unwrap();
    let config_content = r#"
[commands.git]
command = "git"

[commands.nonexistent12345]
command = "nonexistent12345abc"
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;
    let (pass, fail, warn) = checkup::runner::summarize(&results);

    assert!(pass >= 1, "at least git should pass");
    assert!(fail >= 1, "nonexistent command should fail");
    assert_eq!(warn, 0, "no warnings expected");
    assert_eq!(pass + fail + warn, results.len());
}

#[tokio::test]
async fn test_fix_all_no_failures() {
    let dir = tempfile::tempdir().unwrap();
    // Config minimal que passa
    let config_content = r#"
[commands.git]
command = "git"
"#;
    fs::write(dir.path().join("checkup.toml"), config_content).unwrap();

    let config = checkup::config::load_config(&dir.path().join("checkup.toml")).unwrap();
    let ctx = test_context(dir.path());
    let checks = checkup::config::build_checks(&config, dir.path());
    let results = checkup::runner::run_all(&checks, &ctx).await;

    // Fix all deve retornar vazio quando nao ha falhas
    let fix_results = checkup::runner::fix_all(&checks, &results, &ctx).await;
    assert!(fix_results.is_empty(), "no fixes needed when all pass");
}
