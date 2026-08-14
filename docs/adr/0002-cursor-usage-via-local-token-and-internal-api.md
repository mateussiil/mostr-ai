# Obter uso do Cursor lendo o token local, sem WebView

Supera a ADR-0001. Ao pesquisar a implementação de referência (ai-usagebar, projeto Rust open-source do próprio Akita que inspirou este widget), descobrimos que o Cursor já grava um token de sessão localmente em `%APPDATA%/Cursor/User/globalStorage/state.vscdb` (SQLite, tabela `ItemTable`, chave `cursorAuth/accessToken`) — confirmado como presente na máquina de desenvolvimento. Esse token é reutilizável como cookie (`WorkosCursorSessionToken`) contra a API interna `https://cursor.com/api/usage-summary`.

Isso torna a coleta do Cursor estruturalmente idêntica à do Claude: ler uma credencial já gravada localmente pelo próprio app oficial, e chamar um endpoint com ela. Não há necessidade de WebView embutida, login manual, nem persistência de cookies pelo nosso app — o Cursor já mantém a sessão válida enquanto o usuário estiver logado no editor.

**Consequências**: ainda é uma API não documentada/interna (mesma natureza de risco de quebra que a ADR-0001 reconhecia), mas elimina inteiramente a superfície de UI de "login" e a complexidade de gerenciar uma WebView e cookies próprios. Se o token expirar ou o usuário deslogar do Cursor, o ring cai em Stale state (já previsto no design) em vez de pedir novo login — não há ação de reautenticação possível pelo nosso app; o usuário resolve logando no Cursor normalmente.
