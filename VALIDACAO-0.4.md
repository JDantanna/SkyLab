# SkyLab 0.4 — validação

## Interface e persistência

Project layers tem grupos, arrasto/retorno à raiz, olhos por camada e objeto, nomes por menu e duplo clique e exclusão reversível. Teste com dados fictícios: camada renomeada, outra camada excluída (12 para 6 linhas; 2 para 1 polígono), Ctrl+Z restaurando os dados. O formato 6 mantém nomes e visibilidade de polígonos; formatos antigos continuam legíveis. Salvar usa extensão `.skylab`; filtros nativos e fila de abertura aceitam a extensão e o registro por usuário inclui ProgID e ícone do projeto.

Help não contém a seção Validation. Painéis testados em 1280×720 e 1600×900: laterais e flutuantes aumentam proporcionalmente. A restauração do tamanho inicial reaplica o arranjo original; teste automatizado protege contra manter as dimensões maximizadas.

## Desempenho

Comparação local com executáveis release da 0.3-r2 e 0.4: polígono fictício de 512 vértices e 500 linhas carregadas, totalizando 40.000 vértices de voo. Cinco medições após aquecimento: mediana de 22,75 ms para 7,19 ms; cerca de 3,16 vezes mais rápido. Payload de 720.847 para 19.886 bytes. Geometria e nomes de todas as linhas geradas comparados sem diferenças. Os números são desse cenário, não uma garantia para todos os levantamentos.

A prévia envia só o polígono ativo, desenha só os segmentos temporários e não recria a malha visual estática. O resultado ainda passa pela validação de linhas; a superfície já validada não é validada outra vez na saída. Testes comparam a prévia leve com a geração persistida e protegem projeção, furos, nomes e limites. Conteúdo de prévia não altera projeto ou histórico.

## Distribuição

Compilação offline, testes Rust/JavaScript, construção da janela nativa e análise Clippy. Instalação, reparo, atualização e desinstalação usam pastas de teste e preservam arquivos pessoais. As associações são verificadas pelo registro de teste do instalador, sem alterar a escolha de aplicativos do usuário durante os testes. A atualização de versões anteriores migra o desinstalador para remover os registros próprios de MIS e SkyLab.

Testes de software não equivalem a homologação operacional no navegador de bordo.
