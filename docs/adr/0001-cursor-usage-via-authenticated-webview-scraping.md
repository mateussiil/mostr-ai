---
status: superseded by ADR-0002
---

# Obter uso do Cursor via scraping autenticado em WebView, não API

O Cursor não expõe uma API pública de quota/uso da assinatura (diferente do Claude, que permite ler o token OAuth já gravado localmente pelo CLI oficial). Decidimos coletar o dado de uso do Cursor fazendo scraping autenticado da página de billing (cursor.com/settings) através de uma WebView embutida no app.

A sessão é estabelecida por login manual do usuário, uma única vez, na WebView embutida — os cookies resultantes são persistidos localmente e reusados nas coletas seguintes. Rejeitamos a alternativa de pedir login/senha (ou automatizar o login) diretamente no app: isso exigiria guardar uma credencial sensível da conta e automatizar o processo de autenticação, o que se parece mais com comportamento de bot e aumenta o risco de a conta ser sinalizada ou bloqueada pelo Cursor. Login manual do usuário, com reuso de sessão, é indistinguível de uso normal do navegador.

**Consequências**: essa integração é frágil por natureza — qualquer mudança no HTML/layout da página de billing do Cursor, ou expiração/invalidação da sessão, pode quebrar a coleta sem aviso prévio. Isso é aceito como custo do v1; se o Cursor lançar uma API oficial de uso no futuro, essa decisão deve ser revisitada.
