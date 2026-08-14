# Menu de configurações em HTML, não menu nativo do Windows

A primeira versão do menu de configurações (autostart, sair) usava `window.popup_menu()` do Tauri, a API nativa de menu de contexto. Na prática, o menu abria e fechava sozinho quase instantaneamente ao segurar `Alt` + clique direito.

Causa raiz: o Windows sintetiza um evento de "Alt solto" assim que um menu começa a abrir (é assim que ele encerra o "modo de navegação por menu" do teclado), mesmo com a tecla fisicamente pressionada. Nosso watcher de Alt detectava isso e re-travava o overlay (click-through) no meio da interação, matando o menu nativo — que depende da janela continuar interativa. Há também problemas conhecidos e documentados do Tauri v2 no Windows especificamente com `popup_menu` (cliques não registrando, fechamento inesperado).

Substituímos por um menu construído em HTML/CSS dentro do próprio overlay — a mesma tecnologia dos rings, que já sabíamos ser confiável. Isso elimina a dependência do comportamento do Windows em torno de Alt e menus nativos. Para sustentar a interação (já que soltar o Alt pra navegar o menu com o mouse ainda é o fluxo natural), o backend expõe um comando `pin_interactive` que o frontend chama ao abrir/fechar o menu, mantendo a janela interativa nesse intervalo independente do estado real do Alt.

**Consequências**: o menu não se parece 100% com um menu nativo do Windows (sem sombra do sistema, sem navegação por teclado nativa) — aceito, já que o visual é totalmente controlado por nós e pode ser ajustado livremente. Se o Tauri corrigir o comportamento de `popup_menu` no Windows no futuro, essa decisão pode ser revisitada, mas não há urgência: a versão HTML funciona bem.
