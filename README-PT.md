# 🏥 checkup

> Diagnóstico e auto-correção de ambiente para máquinas de desenvolvimento.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![CI](https://github.com/FranciscoBSpadaro/checkup/actions/workflows/ci.yml/badge.svg)](https://github.com/FranciscoBSpadaro/checkup/actions/workflows/ci.yml)

Pare de perder horas debugando sua máquina de dev. Um comando te diz
**exatamente** o que está errado — e como corrigir.

**🇺🇸 [English version](README.md)**

```bash
$ checkup
```

```
┌──────────────────────────────────────────────────────┐
│  checkup v0.1.1 — Diagnóstico de Ambiente            │
├──────────────────────────────────────────────────────┤
│                                                      │
│  ✓ Node.js          v20.10.0 (>= 20)                 │
│  ✓ PostgreSQL       rodando em :5432                 │
│  ✗ Docker           não está rodando                 │
│    └─ Fix: systemctl start docker                    │
│  ✗ Porta 3000        já está em uso                  │
│    └─ Fix: kill $(lsof -t -i:3000)                   │
│  ✗ .env             faltando: DATABASE_URL, JWT_SECRET│
│                                                      │
│  Resumo: 3 problemas encontrados, 2 passaram         │
│  Execute 'checkup fix' para auto-corrigir            │
└──────────────────────────────────────────────────────┘
```

---

## Índice

- [Por quê?](#por-quê)
- [Instalação](#instalação)
- [Início Rápido](#início-rápido)
- [Configuração](#configuração)
- [Checks](#checks)
- [Auto-Fix](#auto-fix)
- [Roadmap](#roadmap)
- [CI/CD](#cicd)
- [Contribuindo](#contribuindo)
- [Licença](#licença)

---

## Por quê?

Todo desenvolvedor já passou por isso:

```bash
$ npm install
Error: node-gyp build failed
# ... 30 minutos depois ...
# Ah, versão errada do Node.
# ... 20 minutos depois ...
# Ah, Docker não tá rodando.
# ... 15 minutos depois ...
# Ah, DATABASE_URL não tá configurada.
```

**3 problemas não relacionados. 65 minutos desperdiçados.**

O `checkup` diagnostica todo o ambiente de dev em **segundos**. Defina o que seu
projeto precisa, rode um comando, veja tudo que está errado — com a correção
logo ao lado.

---

## Instalação

### Via código fonte

```bash
git clone https://github.com/FranciscoBSpadaro/checkup-cli-rust
cd checkup
cargo install --path .
```

### Script de instalação rápida

```bash
curl -fsSL https://raw.githubusercontent.com/FranciscoBSpadaro/checkup-cli-rust/main/scripts/install.sh | bash
```

### Suporte por Plataforma

| Plataforma | Status       |
| ---------- | ------------ |
| Linux      | ✅ Suportado |
| macOS      | ✅ Suportado |

---

## Início Rápido

**Passo 1:** Inicialize a configuração na raiz do projeto:

```bash
checkup init
```

Isso cria um arquivo `checkup.toml` com padrões sensatos.

**Passo 2:** Edite `checkup.toml` para seu projeto:

```toml
[commands.node]
command = "node"
fix = "fnm install 20"

[versions.node]
version_flag = "--version"
expected = ">=20"
fix = "fnm install 20"

[services.postgres]
port = 5432

[ports]
free = [3000, 8080]

[env]
required = ["DATABASE_URL", "JWT_SECRET"]

[envfile]
path = ".env"
required = ["DATABASE_URL", "JWT_SECRET"]
```

**Passo 3:** Rode o diagnóstico:

```bash
checkup
```

**Passo 4:** Auto-corrigia o que for possível:

```bash
checkup fix
```

---

## Configuração

O arquivo `checkup.toml` define o que verificar:

```toml
# Comandos que devem existir no PATH
[commands.node]
command = "node"
fix = "fnm install 20"

# Verificação de versão (constraints semver: >=, >, ==, <, <=)
[versions.node]
version_flag = "--version"
expected = ">=20"
fix = "fnm install 20"

# Serviços (verificação TCP)
[services.postgres]
port = 5432

[services.redis]
port = 6379

# Portas que devem estar livres
[ports]
free = [3000, 8080]

# Variáveis de ambiente requeridas
[env]
required = ["DATABASE_URL", "JWT_SECRET", "REDIS_URL"]

# Validação do arquivo .env
[envfile]
path = ".env"
required = ["DATABASE_URL", "JWT_SECRET"]
```

### Opções de Linha de Comando

```
checkup                    Executa todos os checks (padrão)
checkup init               Cria checkup.toml com ferramentas detectadas
checkup check              Executa todos os checks
checkup fix                Auto-corrige falhas
checkup list               Lista checks configurados
checkup completions        Gera shell completions

Opções:
  -c, --config <FILE>   Caminho para arquivo de config [padrão: checkup.toml]
      --json            Output em formato JSON
  -q, --quiet           Mostra apenas falhas
  -h, --help            Mostra ajuda
  -V, --version         Mostra versão
```

---

## Checks

Cada check verifica um aspecto do ambiente:

| Check     | O que faz                                         |
| --------- | ------------------------------------------------- |
| `command` | Verifica se comando existe no PATH                |
| `version` | Verifica versão da ferramenta (comparação semver) |
| `service` | Verifica se serviço está rodando (TCP port check) |
| `port`    | Verifica se porta está livre para uso             |
| `envfile` | O `.env` existe e tem as chaves necessárias?      |
| `envvar`  | Variáveis de ambiente estão definidas?            |

Checks rodam **em paralelo** usando I/O async — tipicamente completando em menos de 1 segundo.

---

## Auto-Fix

O subcomando `fix` tenta resolver problemas automaticamente:

```bash
$ checkup fix
```

```
Tentando auto-correção para 3 problemas...

  [1/3] Docker: iniciando serviço...             ✓ Feito
  [2/3] .env: criando a partir de .env.example...  ✓ Feito
  [3/3] Porta 3000: matando processo...           ✗ Permissão negada (precisa sudo)

  2/3 corrigidos automaticamente. 1 requer ação manual.
  Execute: sudo kill $(sudo lsof -t -i:3000)
```

Quando auto-fix não é possível, a ferramenta fornece o comando exato para rodar.

---

## Roadmap

- ✅ Estrutura do projeto e arquitetura
- ✅ Core: trait `Check` + plugin system
- ✅ Checks: command, version, service, port, envfile, envvar
- ✅ Execução paralela com async (`futures::join_all`)
- ✅ Parser de config `checkup.toml` (serde)
- ✅ Sugestões de auto-fix
- ✅ Modo output JSON (`--json`)
- ✅ CLI com clap (`init`, `check`, `fix`, `list`, `completions`)
- ✅ Shell completions (bash, zsh, fish, powershell, elvish)
- ✅ 38 testes passando (19 unitários + 10 integração)
- 🚧 Suporte a Windows (pendente):
    - Substituir chamadas `sh -c` por compilação condicional `cfg(unix)` / `cfg(windows)` em `port.rs`, `command.rs`, `version.rs`
    - Substituir `lsof` / `ss` em `port.rs::find_process_on_port()` por `netstat -ano` no Windows
    - Adicionar job Windows no CI rodando `cargo test` (atualmente só executa `build-windows`, sem testes)
    - Adicionar docs de instalação Windows e reabilitar `scripts/install.ps1` quando pronto

---

## CI/CD

Este projeto usa **GitHub Actions** para integração e entrega contínua. Toda push e pull request aciona o pipeline de CI automaticamente.

### Estágios do Pipeline

| Estágio    | Descrição                                                      |
| ---------- | -------------------------------------------------------------- |
| **Check**  | `cargo check` — verificação rápida de compilação               |
| **Format** | `cargo fmt` — garante estilo consistente de código             |
| **Lint**   | `cargo clippy` — captura erros comuns                          |
| **Test**   | `cargo test` — executa todos os testes (38 passando)           |
| **Build**  | `cargo build --release` — artefato de produção                 |

### Plataformas Suportadas (CI)

| Plataforma | Status     |
| ---------- | ---------- |
| Linux      | ✅ Testado |
| macOS      | ✅ Testado |

O pipeline de CI roda em **Linux e macOS** em cada push. Veja [`.github/workflows/ci.yml`](.github/workflows/ci.yml) para detalhes.

---

## Contribuindo

Contribuições são bem-vindas! Veja [CONTRIBUTING.md](CONTRIBUTING.md) para diretrizes detalhadas sobre como reportar problemas, propor funcionalidades e abrir pull requests.

---

## Licença

Este projeto está licenciado sob a [Licença MIT](LICENSE).
