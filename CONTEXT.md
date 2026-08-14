# AI Usage Overlay

Um HUD sempre visível na área de trabalho que mostra, de relance, quanto de uso de cada assistente de IA (Claude, Cursor, Codex) já foi consumido e quando o limite reseta.

## Language

**Overlay**:
O painel sempre visível, semi-transparente e click-through que exibe o estado de uso das IAs sobre a área de trabalho.
_Avoid_: Widget (ambíguo com os Widgets nativos do Windows 11), dashboard, HUD (usar só como sinônimo ocasional, não como termo canônico).

**Provider**:
Um serviço de IA rastreado pelo overlay (Claude, Cursor, Codex). Cada provider tem seu próprio chip no overlay.
_Avoid_: IA, assistente, ferramenta.

**Ring**:
O indicador circular de progresso de um provider. O preenchimento do ring comunica tanto o % de uso consumido quanto, por associação visual, a proximidade do reset — é o elemento central do overlay, não uma barra de progresso comum.
_Avoid_: Progress bar, gauge genérico, donut chart.

**Reset**:
O momento em que um provider renova a cota de uso do usuário. Cada ring mostra o tempo até o próximo reset.
_Avoid_: Refresh (reservado para a atualização dos dados do overlay, ver Refresh), renovação, recarga.

**Refresh**:
O ato do overlay buscar dados novos de uso junto a um provider (polling). Não deve ser confundido com Reset, que é o evento do lado do provider.
_Avoid_: Reset, atualização (usar Refresh como termo canônico).

**Critical state**:
O estado visual de um ring quando o uso de um provider ultrapassa 90% — o ring e o texto de reset mudam de cor para sinalizar urgência.
_Avoid_: Warning, alerta, estado de perigo.

**Unlock gesture**:
A ação de seguar `Alt` para destravar temporariamente a interatividade do overlay (permitindo arrastar, clicar, ou abrir o menu de contexto), já que por padrão ele é click-through.
_Avoid_: Modo de edição, modo admin.

**Stale state**:
O estado de um ring quando o último Refresh falhou (sessão expirada, sem internet, etc.) — mostra o último valor conhecido esmaecido, em vez de sumir ou zerar.
_Avoid_: Estado de erro, offline (reservar para descrever a causa, não o estado visual em si).

**Settings menu**:
O menu construído em HTML dentro do próprio overlay (não é um menu nativo do Windows — essa abordagem foi abandonada por instabilidade), aberto com clique direito durante o Unlock gesture. Contém o toggle de autostart e a opção de sair.
_Avoid_: Menu de contexto nativo, popup menu.

**Hide (ring)**:
A ação de ocultar um Ring específico até o overlay ser reiniciado, via o "x" que aparece ao passar o mouse sobre um Provider durante o Unlock gesture. Não é persistido — reiniciar o overlay traz todos os rings de volta.
_Avoid_: Remover, desativar (esses termos sugeririam algo permanente, que não é o caso).

**Fit-to-content**:
O comportamento do Overlay de sempre ocupar exatamente a largura necessária pros Rings visíveis no momento (sem espaço morto lateral), reajustando automaticamente sempre que um Ring é ocultado. A borda direita fica fixa — o painel encolhe/cresce "pra dentro" a partir da esquerda, preservando onde o usuário posicionou o overlay.
_Avoid_: Auto-resize genérico (o termo canônico já implica a regra de "encolhe pela esquerda, direita fixa").
