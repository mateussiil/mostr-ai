# mostr-ai

Um overlay sempre visível na área de trabalho que mostra, de relance, quanto do limite de uso do **Claude**, do **Cursor** e do **Codex** você já consumiu e quanto tempo falta para cada cota resetar.

Sem abrir navegador, sem entrar em página de billing: um painel pequeno e semi-transparente no canto da tela, com um ring de progresso por provider.

## Como funciona

- **Um ring por provider**: o preenchimento mostra o % de uso consumido, e embaixo aparece o tempo até o próximo reset.
- **Critical state**: acima de 90% de uso, o ring muda de cor para chamar atenção.
- **Stale state**: se um refresh falhar (sem internet, sessão expirada), o ring continua mostrando o último valor conhecido, esmaecido, em vez de sumir ou zerar.
- **Refresh automático** a cada 5 minutos.
- **Click-through por padrão**: o overlay nunca rouba cliques do que está embaixo dele.
- **Some sozinho em tela cheia** (jogos, vídeos, apresentações) e volta quando você sai.
- **Ajusta a largura ao conteúdo**: ocultar um ring encolhe o painel pela esquerda, mantendo a borda direita onde você deixou.

### Interagindo com o overlay

Segure **`Alt`** para destravar o overlay temporariamente. Com o `Alt` pressionado você pode:

| Ação | Resultado |
| --- | --- |
| Arrastar | Move o overlay. A posição fica salva entre execuções |
| Passar o mouse num provider e clicar no **x** | Oculta aquele ring até o app reiniciar |
| Clique direito | Abre o menu de configurações (iniciar com o Windows, sair) |

## De onde vêm os dados

O mostr-ai **não pede login nem senha**. Ele lê as credenciais que os apps oficiais já gravam na sua máquina e consulta os endpoints de uso de cada serviço:

| Provider | Credencial lida localmente | Endpoint |
| --- | --- | --- |
| Claude | `~/.claude/.credentials.json` (Claude Code CLI) | `api.anthropic.com/api/oauth/usage` |
| Cursor | `%APPDATA%/Cursor/User/globalStorage/state.vscdb` | `cursor.com/api/usage-summary` |
| Codex | `~/.codex/auth.json` (Codex CLI) | `chatgpt.com/backend-api/wham/usage` |

Ou seja: basta estar logado no Claude Code, no Cursor e/ou no Codex CLI. Se você deslogar de algum deles, o ring correspondente entra em stale state, e para resolver basta logar de novo na ferramenta original.

O token do Codex é renovado **só em memória**. O mostr-ai nunca escreve nos arquivos de login de outras ferramentas (veja a [ADR-0003](docs/adr/0003-codex-token-refresh-in-memory-only.md)).

> ⚠️ Os endpoints do Cursor e do Codex são internos e não documentados. Podem mudar sem aviso e quebrar a coleta daquele provider.

## Plataforma

Feito para **Windows**. A detecção da tecla `Alt` e de apps em tela cheia usa APIs Win32. Em outros sistemas o app compila, mas essas funcionalidades ficam desativadas.

## Stack

- [Tauri 2](https://tauri.app/) com backend em Rust (`reqwest`, `tokio`, `rusqlite`, crate `windows`)
- Frontend em TypeScript puro + Vite, sem framework

## Rodando localmente

Pré-requisitos: [Node.js](https://nodejs.org/), [Rust](https://rustup.rs/) e os [pré-requisitos do Tauri para Windows](https://tauri.app/start/prerequisites/) (WebView2 e Build Tools do MSVC).

```bash
npm install
npm run tauri dev     # modo desenvolvimento
npm run tauri build   # gera o instalador
```

## Estrutura

```
src/                 frontend (rings, menu de configurações)
src-tauri/src/
  claude.rs          coleta de uso do Claude
  cursor.rs          coleta de uso do Cursor
  codex.rs           coleta de uso do Codex (+ refresh OAuth em memória)
  input.rs           watcher do Alt (click-through ↔ interativo)
  fullscreen.rs      esconde o overlay quando há app em tela cheia
  config.rs          posição e autostart persistidos
  lib.rs             janela, comandos e loop de refresh
docs/adr/            decisões de arquitetura
CONTEXT.md           glossário do domínio (overlay, ring, reset, stale state…)
```

## Documentação

- [`CONTEXT.md`](CONTEXT.md): vocabulário canônico do projeto
- [`docs/adr/`](docs/adr/): por que as coisas são como são (scraping vs. token local, refresh em memória, menu HTML vs. nativo)
