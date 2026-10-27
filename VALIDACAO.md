# Validação local — 27/10/2026

## Resultado

Executável Windows de 64 bits compilado em modo release. Interface local em português verificada no navegador, com cálculo Rust. Não foi executado o SkyLab antigo, não houve teste em voo nem abertura dos resultados no SkyMap/equipamento de bordo. Os arquivos de origem foram apenas lidos; nenhum deles foi editado ou substituído. O pacote não contém arquivos reais ou a documentação proprietária extraída.

### Testes automatizados

**24 testes aprovados**: 23 de geometria/formatos/projeção e 1 de integração HTTP que inicia e encerra seu próprio servidor. `cargo test --offline` e `cargo clippy --all-targets --offline -- -D warnings` passaram. A sintaxe JavaScript também foi verificada com `node --check`.

Cobertura: vetores UTM publicados pelo PROJ nos dois hemisférios; ida e volta de projeção em quatro zonas e cinco latitudes; rejeição de projeção/hemisfério inválidos e valores não finitos; espaçamento, orientação, sentido alternado e numeração; recorte em polígono côncavo, união de blocos, furos, tangências e bordas; preservação de vértices intermediários; corte por distância acumulada; validação topológica; parâmetros inválidos e estouro de numeração; nomes repetidos/incompatíveis; ida e volta MIS, KML e JSON com precisão de ponto flutuante; recusa de linhas que colapsariam ao arredondar o MIS; XML malformado e entidades externas; entrega da interface/API e recusa de origem externa.

A checagem de projeção com vetores externos usou tolerância de 2 cm contra as coordenadas publicadas com duas casas decimais pelo [PROJ](https://proj.org/en/stable/operations/projections/utm.html). Isso é um teste de implementação, não uma certificação de acurácia global.

### Acervo real

| Verificação | Resultado |
|---|---|
| MIS disponíveis | 6 arquivos, 458 linhas |
| Importar → exportar → reimportar MIS | Nomes e coordenadas idênticos em todos os 6 arquivos |
| Polilinhas reais | Preservados até 16 vértices em uma linha MIS |
| KML pesquisados recursivamente | 53 arquivos |
| KML importados | 52; nomes ausentes/repetidos adaptados com notas explícitas |
| KML recusado | 1 arquivo de zonas proibidas, por exceder o limite de vértices |
| Tipos dos KML aceitos | 49 arquivos de linhas; 3 com polígonos |
| Arquivos poligonais | 2 de levantamento e 1 de zonas proibidas; não são semanticamente intercambiáveis |

Comparações de pares selecionados **sem presumir equivalência por nome de arquivo**:

| Par local | Linhas MIS / KML | Geometrias MIS com correspondência KML a menos de 1 m |
|---|---:|---:|
| A | 67 / 67 | 67 |
| B | 54 / 57 | 51 |
| C | 140 / 150 | 130 |

Esses pares não tinham nomes de linha coincidentes. A comparação geométrica considerou mesmo número de vértices e ordem direta ou invertida. Nos pares B/C há diferenças de versão/edição que impedem afirmar equivalência integral. A coincidência geométrica sustenta o uso de WGS84/UTM 23S nesses casos; não prova o datum dos demais arquivos.

### Geração em blocos reais

Parâmetros de teste, sem finalidade operacional: linhas de missão a 100 m / azimute 120° / fase 50 m; linhas de controle a 500 m / azimute 30° / fase 250 m; mínimo 50 m; prefixos e numerações separados.

| Bloco | Partes de missão | Partes de controle | Extensão total de aquisição |
|---|---:|---:|---:|
| A | 337 | 59 | 4.950,82 km |
| B | 447 | 84 | 4.192,33 km |

Ambos passaram por geração, acréscimo de malha de controle, exportação MIS/KML e reabertura JSON. A extensão soma segmentos; não inclui deslocamentos, curvas ou conexões.

- Maior diferença por vértice em MIS após arredondamento: **0,00688 m**.
- Maior diferença de ida e volta KML: **0,000052 m**, como erro numérico interno; não representa acurácia de campo.
- Projeto JSON reaberto com geometria, nomes, blocos, notas e projeção idênticos. Uma diferença inicial nos últimos bits de ponto flutuante motivou ativar leitura JSON com precisão de ida e volta e criar um teste de regressão.

### Experiência e interface

Foram consultados o manual CHM e capturas nele contidas, sem executar o binário antigo. O fluxo antigo exigia selecionar a camada/polígono e abrir uma janela separada de geração, com distância, ângulo, numeração, centro X/Y e botões Generate/Reload/Apply. As páginas específicas de opções e corte estavam incompletas no manual.

A interface nova apresenta quatro etapas visíveis, configuração ao lado da prévia, área de revisão com busca, seleção sincronizada e edição contextual. A projeção permanece explícita e bloqueada enquanto há um projeto, evitando reinterpretar as mesmas coordenadas em outra zona. Novo projeto e desfazer permitem retornar. Exportação reúne contagem, extensão, projeção e formato.

Verificados visualmente em tela ampla e estreita: bloco fictício com furo, linhas interrompidas pela exclusão, seleção destacada, sentido, escala, grade e organização responsiva. Foi corrigida a altura do SVG no modo estreito para usar toda a área disponível.

Fluxo interativo verificado: demonstração com 23 partes; renomeação; corte de 100 m a 1.000 m, produzindo 900 m; divisão em dois trechos de 450 m com nomes próprios (24 partes no total); desfazer/refazer; revisão da exportação e acionamento do download MIS. As operações usaram somente geometria fictícia. A imagem `interface.png` registra a demonstração, não o acervo confidencial.

## Decisões necessárias antes de uso operacional

1. Confirmar **datum, zona e hemisfério** usados em cada missão e no navegador de bordo. Esta versão suporta apenas WGS84/UTM, com as restrições descritas no README.
2. Confirmar o **modelo/versão do navegador e sua aceitação de MIS**: polilinhas, nomes com sufixos, limites de nome e convenção de numeração. A estrutura textual foi comparada com os exemplos; compatibilidade real a bordo ainda não foi demonstrada.
3. Definir a convenção da equipe para linhas de missão/controle, primeira linha, incremento, sentido e fase da malha. A fase relativa à borda é diferente do centro X/Y do programa antigo.
4. Preparar zonas de exclusão como furos válidos em SIG. Polígonos independentes de zonas proibidas não são subtraídos automaticamente. O arquivo muito detalhado recusado precisa de tratamento externo adequado.
5. Validar coordenadas e resultados antes de qualquer uso operacional em voo e acrescentar as verificações de terreno, obstáculos, restrições e manobras exigidas pelo procedimento da operação.

Não foram feitas publicações, uploads de dados, modificações do navegador de bordo ou integração com sistemas de voo.
