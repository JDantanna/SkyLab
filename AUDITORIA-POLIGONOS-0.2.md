# Revisão do código — polígonos, SkyLab 0.2, revisão interna 3

Três instâncias independentes revisaram o código, sem editar simultaneamente: cálculos/API Rust; interface JavaScript/CSS/HTML; integração Windows/instalação/atualização. Achados foram conferidos e implementados no fluxo principal.

Removidos: install_from_executable, SetupSource e seu ramo de cópia por arquivo (o instalador real usa o payload em memória); teste dedicado somente a essa API morta; handlers de arquivo, geração e Novo projeto sobrescritos; nove linhas de tradução obsoletas/duplicadas; regra de altura e classe de layout substituídas.

Otimizado: auxiliares públicos continuam validando entradas, sem repetir a mesma validação no despachante. Inserção de âncora usa caminho interno já validado. O limite de anéis/curvas vem da mesma constante. Camadas preservam o formato de projeto ao recortar exportações. Permanecem limites de geometria e descarte de respostas antigas de prévia.

Regressões corrigidas: selecionar linhas por retângulo limpa seleção de polígono antes de Excluir; Excluir aceita polígono pela barra; prévia XYZ é redesenhada após conversão geográfica; camadas com curvas exportam JSON e KML; cancelamento/erro de arrasto restaura a visualização; rolagem fica bloqueada durante arrasto de ponto. Coordenadas XYZ passam efetivamente a bottom:0, junto da atribuição.

A cadeia de wrappers restante foi mantida porque possui consumidores reais e preserva desfazer, abas, satélite e recuperação. Nenhuma reescrita ampla foi feita apenas para reduzir contagem de linhas.
