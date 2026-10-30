# Revisão e otimização — SkyLab 0.5, revisão interna 2

Versão visível mantida em 0.5, conforme aprovação. Revisão de código próprio Rust, interface, janela nativa, instalador, scripts de associação e atualização. Bibliotecas e dados de projeção preservados; nenhuma nova dependência.

## Mudanças verificáveis

- Pontos de edição: leitura envia somente a linha ou polígono selecionado, com CRS e metadados de camadas. Validação da geometria solicitada permanece no Rust; operações que alteram o projeto continuam validando o projeto completo.
- Cartografia: chamadas redundantes de redesenho compartilham um quadro; cancelamento/saída remove trabalho pendente. Controles fora da janela deixam de criar elementos invisíveis; alças visíveis continuam acessíveis.
- Tradução ignora alterações nos elementos gráficos transitórios e no texto da cartografia que já é traduzido explicitamente. Menus, mensagens e demais controles continuam traduzidos.
- Contornos retos usam segmentos SVG diretos, evitando projeções repetidas. Curvas continuam usando seus controles Bézier.
- Linhas e polígonos compartilham interpolação, subdivisão e amostragem Bézier. Tolerância de 0,1 m, profundidade máxima e limites individuais preservados.
- Inserção de ponto elimina revalidação intermediária; emenda elimina cópias descartadas e valida somente entrada e resultado final. Histórico e dados de origem permanecem imutáveis.
- Nomes DRAW_n passam a consultar um conjunto indexado, evitando procurar em todas as linhas para cada número. Camada existente evita cálculo desnecessário de novo ID.
- Importação QGIS conta cada fonte uma vez durante a leitura, mantendo limites de recursos e conferência final dos dados importados.

## Medição reproduzida com dados sintéticos

10.000 linhas, 200.000 vértices; leitura dos 20 pontos de uma linha selecionada. Duas execuções de aquecimento e mediana de sete medições no mesmo computador e servidor local da versão anterior.

| Medida | Antes | Depois |
|---|---:|---:|
| Pedido de leitura | 3,779,092 bytes | 577 bytes |
| Tempo mediano | 55.09 ms | 0.60 ms |

Redução de 99,98% no tamanho desse pedido. O ganho se refere à leitura de pontos selecionados, não à velocidade global do programa. Nenhum vértice foi simplificado ou removido.

## Validação

- Compilação completa com scripts/build.ps1: testes de interação, geometria, importação QGIS, histórico, CRS, arquivos, atualização, instalação e inicialização nativa aprovados.
- Análise Rust com todas as ferramentas e instalador habilitado, sem avisos.
- 36 comparações com o executável público 0.5-r1: desenho curvo, inserção e emenda com resultados geométricos idênticos.
- Testes adicionais verificam consolidação/cancelamento de quadros, isolamento de geometria e redução de projeções dos contornos retos.
- Busca de funções JavaScript declaradas sem referências não encontrou candidatas adicionais. Recursos carregados dinamicamente, bibliotecas de projeção, associações e mecanismos de recuperação foram preservados.

A redução de código concentra-se em duplicações e trabalho repetido comprovados. A revisão não constitui prova de ausência de todos os defeitos nem substitui validação operacional.
