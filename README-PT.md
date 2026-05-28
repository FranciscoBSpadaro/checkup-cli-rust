# 🏥 checkup

> Diagnóstico e auto-correção de ambiente para máquinas de desenvolvimento.

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Pare de perder horas debugando sua máquina de dev. Um comando te diz
**exatamente** o que está errado — e como corrigir.

**🇺🇸 [English version](README.md)**

```bash
$ checkup
```

```
┌──────────────────────────────────────────────────────┐
│  checkup v0.1.0 — Diagnóstico de Ambiente            │
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
│  Execute 'checkup --fix' para auto-corrigir          │
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
git clone https://github.com/seuuser/checkup.git
cd checkup
cargo install --path .
```

### Suporte por Plataforma

| Plataforma | Status |
|------------|--------|
| Linux      | ✅ Suportado |
| macOS      | ✅ Suportado |
| Windows    | 🚧 Planejado |

---

## Início Rápido

**Passo 1:** Inicialize a configuração na raiz do projeto:

```bash
checkup init
```

Isso cria um arquivo `.checkup.toml` com padrões sensatos.

**Passo 2:** Edite `.checkup.toml` para seu projeto:

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
checkup --fix
```

---

## Configuração

O arquivo `.checkup.toml` define o que verificar:

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
checkup [OPTIONS]

Opções:
  -c, --config <FILE>   Caminho para arquivo de config [padrão: .checkup.toml]
  -f, --fix             Tenta auto-corrigir problemas
  -j, --json            Output em formato JSON
  -q, --quiet           Mostra apenas falhas
      --init            Cria .checkup.toml padrão
  -h, --help            Mostra ajuda
  -V, --version         Mostra versão
```

---

## Checks

Cada check verifica um aspecto do ambiente:

| Check | O que faz |
|-------|-----------|
| `command` | Verifica se comando existe no PATH |
| `version` | Verifica versão da ferramenta (comparação semver) |
| `service` | Verifica se serviço está rodando (TCP port check) |
| `port` | Verifica se porta está livre para uso |
| `envfile` | O `.env` existe e tem as chaves necessárias? |
| `envvar` | Variáveis de ambiente estão definidas? |

Checks rodam **em paralelo** usando I/O async — tipicamente completando em menos de 1 segundo.

---

## Auto-Fix

A flag `--fix` tenta resolver problemas automaticamente:

```bash
$ checkup --fix
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

- [x] Estrutura do projeto e arquitetura
- [x] Core: trait `Check` + plugin system
- [x] Checks: command, version, service, port, envfile, envvar
- [x] Execução paralela com async (futures::join_all)
- [x] Parser de config `.checkup.toml` (serde)
- [x] Sugestões de auto-fix
- [x] Modo output JSON (`--json`)
- [x] CLI com clap (init, check, --fix, --quiet)
- [x] 18 testes unitários passando
- [ ] Shell completions (bash, zsh, fish)
- [ ] Suporte a Windows

---

## Contribuindo

Contribuições são bem-vindas! Veja [CONTRIBUTING.md](CONTRIBUTING.md) para diretrizes.

```bash
# Fork e clone
git clone https://github.com/youruser/checkup.git
cd checkup

# Rodar testes
cargo test

# Rodar linter
cargo clippy

# Formatar código
cargo fmt

# Build
cargo build --release
```

---

## Licença

Este projeto está licenciado sob a [Licença MIT](LICENSE).
