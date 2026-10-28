# Validação — SkyLab 0.2, revisão interna 3

- 67 testes Rust e 27 testes de interação passaram, além dos dois cenários nativos de construção de janela/logo. Compilação completa offline do aplicativo e instalador e clippy com warnings recusados.
- Curvas: JSON preserva âncoras/alças, inserção mantém contorno, parâmetros não lineares são respeitados, autointerseções/NaN e metadados inconsistentes são recusados. Camadas com curvas exportam JSON e KML; furos são preservados. Reprojeção lineariza e registra uma nota.
- Geração por índice de polígono mantém os demais polígonos e suas camadas. Prévia não entra no histórico.
- Conferência visual em 1280×720, somente dados sintéticos: desenho fechado com Bézier, seleção na árvore, arrasto de ponto, Shift inserindo uma quinta âncora e Ctrl+Z restaurando quatro; geração de seis linhas; retângulo selecionando seis linhas e ocultando a barra do polígono; Delete removeu somente as seis linhas, manteve o polígono e Ctrl+Z restaurou as seis.
- Projeto com curvas foi salvo/reaberto e recuperado pelo fluxo de recuperação, mantendo pontos, alças, seleção do polígono e campos. Traçado temporário permite Ctrl+Z sem alterar o polígono existente; Escape descarta o traçado.
- XYZ Google: prévia apareceu ao concluir cálculo, sem mover o mapa. Coordenadas e atribuição medidas na borda inferior, sem sobreposição com o zoom.
- Três instâncias independentes revisaram Rust, interface e integração Windows. Limpeza e correções detalhadas em AUDITORIA-POLIGONOS-0.2.md.
- Instalação, reparo e desinstalação de executáveis reais em pasta isolada preservaram projeto/configuração e recusaram Origin externo. Atualizações 0.1-r6 → 0.2-r3 e 0.2-r2 → 0.2-r3 verificaram SHA256/identidade, reiniciaram sem setup e restauraram três abas, seleção, enquadramento, parâmetros e estado modificado. Arquivos pessoais e atalhos foram preservados.
- Associação MIS real no Explorer permanece limitada pelas permissões do ambiente; o registro simulado e sua remoção passam nos testes. Não houve nova alteração desse recurso.
- Dados de voo reais não foram incluídos nem enviados ao repositório. Testes de software não constituem homologação no navegador de bordo.
