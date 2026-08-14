# Renovar o token OAuth do Codex em memória, sem gravar em auth.json

Diferente do Claude e do Cursor (cujos tokens locais não exigiam renovação para as chamadas do overlay), o access token do Codex em `~/.codex/auth.json` expira e precisa ser renovado via OAuth (`POST https://auth.openai.com/oauth/token`, `grant_type=refresh_token`) antes de chamar o endpoint de uso.

Decidimos fazer essa renovação inteiramente em memória — usar o novo `access_token` só para a chamada de uso da sessão atual, e nunca escrever de volta em `auth.json`. Consideramos persistir o token renovado (mantendo `auth.json` sincronizado, como o próprio Codex CLI faria), mas rejeitamos: esse arquivo é o estado de login de uma ferramenta de terceiros que não controlamos: um erro de formatação, uma corrida entre processos (o Codex CLI rodando ao mesmo tempo), ou uma mudança futura no schema do arquivo poderiam corromper o login real do Codex — um risco desproporcional ao benefício de economizar uma chamada OAuth ocasional.

**Consequências**: cada ciclo de Refresh do overlay pode custar uma chamada extra ao endpoint de OAuth se o token estiver expirado (em vez de reusar um token já renovado por uma leitura anterior). Isso é aceito — é barato e não tem efeito colateral algum sobre o estado do Codex CLI.
