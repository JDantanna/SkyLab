# Validação local — 27/10/2026

## Resultado

Executável Windows de 64 bits compilado em modo release. Interface local em português verificada no navegador, com cálculo Rust. Não foi executado o SkyLab antigo, não houve teste em voo nem abertura dos resultados no SkyMap/equipamento de bordo. Os arquivos de origem foram apenas lidos; nenhum deles foi editado ou substituído. O pacote não contém arquivos reais ou a documentação proprietária extraída.

### Testes automatizados

**28 testes aprovados na versão 0.2.0**: 23 de geometria/formatos/projeção, 1 de integração HTTP que inicia e encerra seu próprio servidor e 4 de instalação. `cargo test --offline` e `cargo clippy --all-targets --features installer --offline -- -D warnings` passaram após gerar o payload. A sintaxe JavaScript do aplicativo e do instalador também foi verificada com `node --check`.

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

## Instalador e identidade — versão 0.2.0

A logo exata foi recuperada do recurso About do SkyLab original, sem executar esse binário. A marca aparece no aplicativo, na tela Sobre e no instalador. Azul-escuro e cinzas da marca orientam a paleta; o instalador compartilha os estilos do aplicativo. Ícone e versão Windows foram incorporados e lidos de volta nos dois executáveis.

Os quatro novos testes verificam instalação/atualização/desinstalação, destinos de atalhos reais `.lnk`, preservação de arquivos pessoais, recusa de pasta desconhecida, recusa de payload inválido e bloqueio de atualização/desinstalação enquanto o aplicativo está em execução, antes de modificar os registros. Em modo de teste, a descrição do registro é gravada em JSON dentro da pasta isolada; o registro e os atalhos pessoais do Windows não são alterados.

Também foi executado o instalador completo com o payload real em pasta de teste. A interface mostrou as fases de preparação e conclusão. Em seguida foi executado o desinstalador instalado, que removeu o aplicativo, atalhos isolados, metadados e seu próprio executável após encerrar. Um projeto fictício colocado dentro da pasta permaneceu intacto. Não foi instalada uma cópia de uso real no perfil do usuário durante a verificação.

A entrada real em Aplicativos instalados é escrita em HKCU pelo script de registro; esse ramo não foi aplicado ao perfil do usuário nesta sessão. Deve ser conferido numa instalação piloto. O instalador não tem assinatura digital corporativa, não implementa instalação por máquina e não inclui atualizador online. A janela usa Edge em modo aplicativo quando disponível, com navegador padrão como alternativa; não é um host WebView2 incorporado.

O código foi organizado em repositório Git local com versões marcadas e uma cópia bundle para migração. Não há hospedagem remota configurada. A documentação do repositório distingue o espaço de trabalho local de um servidor Git e explica como continuar e migrar o histórico.

Não foram feitas publicações, uploads de levantamentos, modificações do navegador de bordo ou integração com sistemas de voo.


## Revisão institucional atual — versão mantida em 0.1

A marcação preliminar 0.2 deixou de ser a versão corrente por instrução expressa do usuário. A aplicação, o instalador e o registro exibem 0.1. Novos avanços exigem uma pergunta e aprovação explícita; esse procedimento está registrado em AGENTS.md.

O instalador foi simplificado para nome, logo, destino, opções e etapas. Os slogans e o painel promocional foram removidos. A marca agora é o SVG branco fornecido pelo usuário, preservado sem alterações. As barras de rolagem, seleção de campos e controles seguem a paleta azul.

A janela do navegador foi substituída por um host Windows WebView2 com FormBorderStyle.None. Os botões de minimizar, maximizar/restaurar e fechar são controles nativos desenhados no azul, fora da superfície dos diálogos HTML. O adaptador foi compilado com avisos tratados como erros. A janela de teste foi aberta no Windows, mas a conferência manual de minimizar/maximizar não foi concluída: ela estava minimizada e a automação foi interrompida por interação no computador. Essa conferência visual permanece pendente; a implementação não usa a moldura cinza do Edge.

**33 testes automatizados passaram**: 23 de geometria e formatos, 1 da API/interface, 5 de instalação e 4 de consulta de atualização. Também passou a análise estática com avisos tratados como erros. O progresso é fornecido pela operação Rust em segundo plano; a interface foi observada em 'Copiando aplicativo — 35%' e depois em 'Instalação concluída'. Não foi acrescentada espera artificial para simular uma instalação.

O botão Checar atualização foi acionado na interface. Exibiu versão instalada 0.1 e fonte não configurada. Os testes cobrem comparação numérica de versões, manifesto local, ausência de fonte e rejeição de endereços inadequados. Nenhum servidor corporativo foi inventado ou consultado. A consulta HTTPS está implementada com validação de certificados, limite de tamanho e tempo; a integração com o servidor real depende de seu endereço e acesso.

A instalação piloto no registro real do usuário, a assinatura corporativa e a validação operacional a bordo continuam pendentes. O WebView2 Runtime é pré-requisito para a janela Windows e já estava disponível neste computador. Os arquivos reais de levantamento continuam fora do repositório e dos pacotes.

Na retomada da verificação, a janela real foi identificada como SkyLab, hospedada por SkyLab.Window.exe. A acessibilidade do Windows confirmou os controles nativos Minimizar, Maximizar/restaurar e Fechar habilitados, inclusive com o diálogo Sobre aberto e exibindo a versão 0.1. A captura de imagem nativa continuou retornando timeout, e a tentativa de interação automatizada foi bloqueada por entrada do usuário. A aplicação foi deixada aberta para avaliação; não foi forçado foco nem fechamento da janela.

O instalador final 0.1 também passou por instalação, reparo, substituição de uma marcação preliminar 0.2 por 0.1, conferência do hash do aplicativo copiado e desinstalação completa em pasta isolada, preservando projeto fictício e configuração de atualização.

## Revisão de operação — 0.1

- 37 testes automatizados: 23 geometria/formatos, 1 integração HTTP com detecção/declaração/ausência de CRS e reprojeção, 5 instalação, 4 projeções PROJ, 4 consulta de atualização. Clippy com avisos como erros.
- PROJ: ida/volta em SIRGAS 2000/UTM23S, ETRS89/UTM32N, NAD83/UTM15N; grade Minnesota incluída. Recusa comprovada da operação NAD27/Alaska quando falta a melhor grade. Essa recusa é um resultado esperado; as mensagens PROJ nos testes não representam falha da suíte.
- Interface com dados fictícios: seleção Shift de duas linhas, menu contextual, exclusão conjunta e restauração por Ctrl+Z; onze inversões seguidas e onze comandos de desfazer; total de 23 linhas e 41,28 km no exemplo; idioma dinamarquês e volta ao inglês; sete etapas do tutorial; aplicação do catálogo SIRGAS 2000/UTM23S.
- Nenhum arquivo real enviado a serviço externo. Nenhuma chave Google fornecida, criada ou usada. A imagem de satélite real exige validação após a configuração de uma chave da empresa e revisão de suas políticas; não foi testada com imagens reais.
- O catálogo completo não significa que todos os sistemas possam servir de plano de trabalho: sistemas geográficos, verticais, compostos, unidades não métricas, eixos diferentes de leste/norte e definições obsoletas ficam para consulta. Grades regionais adicionais não estão todas incorporadas.
- A versão do produto permanece 0.1 enquanto não houver autorização para avançar. A pergunta foi enviada nesta solicitação; nenhuma resposta de avanço foi recebida até o empacotamento.
- O pacote final passou novamente pelo ciclo completo de instalação, reparo e desinstalação em pasta isolada, com conferência do hash do aplicativo e preservação de projeto e configuração. A remoção do próprio desinstalador aguarda bloqueios transitórios do Windows por até 30 segundos.
- A versão final foi conferida pela interface incorporada: total 41,28 km no exemplo, Sobre sem Encerrar, troca inglês/português e versão 0.1 corretamente traduzida. A prévia tutorial-0.1.png mostra o conteúdo da interface, sem representar a moldura nativa.
- Nesta rodada, as novas janelas iniciadas pelo ambiente de execução não ficaram disponíveis para inspeção na área de trabalho. A janela C# compila com avisos como erros; a inspeção interativa do novo diálogo Salvar do Windows permanece pendente. Não foram encerradas as janelas de trabalho do usuário.

## Revisão de layout, seleção e marca — produto 0.1

- Confirmação explícita do usuário: continuar na versão 0.1.
- 37 testes Rust e 5 testes JavaScript passaram (42 ao todo). O conjunto JavaScript executa o manipulador real de Ctrl+Z e os métodos reais de histórico/importação, com ambiente DOM de teste: exclusão/restauração de geometria, seleção e câmera; onze exclusões/desfazer/refazer; ramificação; limite de 30 estados; enquadramento exclusivo da geometria adicionada.
- Causa da falha de Ctrl+Z: referência inválida `tour-layer.hidden` no código do atalho. Corrigida para leitura explícita do elemento e protegida pelo teste de regressão.
- Conferência da interface com o exemplo fictício: Shift da primeira à quinta linha selecionou as cinco; excluir reduziu 23 para 18 e Ctrl+Z restaurou 23 e a seleção de cinco; clique em espaço vazio limpou todas; clique no mapa selecionou L100300 na lista. Rótulo Adicionar arquivo, bandeiras, troca para português, Sobre pelo nome e exemplo no fim de Ajuda verificados.
- Novo ícone conferido no aplicativo e instalador. Verificação binária confirmou as sete imagens ICO dentro de ambos os executáveis; o caminho do S é idêntico ao SVG original.
- Ciclo do instalador final passou em pasta isolada: instalação, reparo, hash do payload, atalhos de teste, desinstalação com remoção do próprio arquivo e preservação do projeto/configuração. Nenhuma instalação pessoal foi substituída durante os testes.
- Clippy passou com avisos como erros. As capturas layout-0.1.png e instalador-layout-0.1.png mostram o conteúdo das interfaces, sem representar a moldura da janela nativa.
- Pesquisa Google documentada em GOOGLE-MAPS-CUSTOS.md, com fontes oficiais. Sem chave fornecida, sem imagens reais consultadas e sem alteração de faturamento. A integração de satélite real continua dependendo da configuração da empresa.

## Revisão de otimização — 0.1

A limpeza em três frentes foi integrada e validada; consulte AUDITORIA-OTIMIZACAO-0.1.md. Suíte JavaScript e Rust, Clippy e ciclo completo do instalador isolado passaram. Exemplo da interface preserva 23 linhas/41,28 km, incluindo restauração após exclusão por Ctrl+Z. Medianas locais: geração 66,646 → 50,860 ms; validação de 40 mil vértices 69,860 → 56,855 ms. Dados fictícios, sete execuções medidas após aquecimento. Esses números não garantem o mesmo ganho em outros cenários.
