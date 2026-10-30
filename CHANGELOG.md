# Histórico de versões

## 0.5 — revisão de desempenho (revisão interna 2)

- Leitura de pontos de edição envia somente a geometria selecionada, preservando validação de CRS e camadas.
- Desenho da cartografia consolidado por quadro; tradução deixa de percorrer a interface durante atualizações gráficas. Pontos fora da janela não geram controles invisíveis.
- Matemática Bézier compartilhada entre linhas e polígonos, sem mudar tolerância ou limites; edição evita cópias descartadas e validações repetidas.
- Importação QGIS contabiliza cada fonte uma vez, mantendo limites e validação final.

## 0.5 — cartografia e projetos QGIS (revisão interna 1)

- Shift+C abre/fecha o modo cartografia após carregar geometria. Workspace ampliado e barra vertical flutuante com seleção, corte, caneta, mão, excluir, emendar, aplicar/cancelar, satélite, enquadrar e histórico. Layout de painéis preservado ao sair.
- Linhas aceitam controles Bézier persistidos no formato 7: mover pontos e alças, adicionar ponto com Shift, excluir pontos, estender extremos, desenhar linha aberta e emendar com prévia confirmada. Cálculos e validação no Rust; exportação MIS mantém todos os vértices achatados a tolerância de 0,1 m.
- Importação QGS/QGZ/QLR e vetores locais GeoPackage, Shapefile, GeoJSON e KML/KMZ; grupos, visibilidade e CRS preservados/transformados para o sistema de trabalho confirmado. Fontes ausentes, remotas e formatos não suportados são informados.
- Originais QGIS são lidos sem alteração; projetos editados continuam sendo salvos em .skylab. Estilos, layouts de impressão, plugins, rasters e tabelas de atributos ficam fora da importação.
- Aviso de falhas parciais da imagem XYZ acompanha os tiles ativos, sem ser escondido por outro tile carregado com sucesso.


## 0.4 — revisão de código e otimização (revisão interna 4)

- Hierarquia indexada por projeto imutável: filhos, visibilidade herdada e totais de grupos deixam de percorrer toda a geometria para cada camada.
- Mission lines e Project layers reutilizam o índice e agregam contagens sem repetir filtros sobre todas as linhas.
- Importação por arrasto centralizada, removendo manipuladores antigos; fallback inicial de lista e botão satélite sobrescritos foram simplificados.
- Corte Rust não clona linhas para descartá-las logo em seguida e reutiliza a visibilidade por camada durante o corte.
- Preservados formatos, validações geométricas, licenças, bibliotecas de projeção e interface institucional.


## 0.4 — fechamento, título e abas (revisão interna 3)

- Fechar projeto organiza Salvar e fechar (azul), Fechar sem salvar e Voltar em três botões alinhados que ocupam toda a largura.
- Cabeçalho do workspace identifica Projeto e mostra o nome da aba ativa, com reticências e título completo ao passar o cursor. Nomes de projetos não são traduzidos.
- Abas encostam no limite esquerdo e no cabeçalho, sem espaçamentos externos entre seus limites.


## 0.4 — abertura, divisão e painéis (revisão interna 2)

- Importação entre zonas/hemisférios incompatíveis preserva o projeto anterior em sua aba e abre o arquivo no sistema confirmado pelo operador; limite de abas recebe mensagem explícita.
- Erros de abertura aparecem também na tela inicial; arrastar arquivos funciona no centro do workspace e no host nativo.
- Menu de linha abre Cortar por distância, inclusive quando o painel estava fechado ou recolhido.
- Divisão calculada no Rust no limite escolhido, preservando vértices, compartilhando o ponto de divisão e evitando nomes repetidos; ambas as partes ficam selecionadas e têm desfazer.
- Limite de corte editado atualiza a distância de divisão; redesenhos mantêm os valores.
- Setas de cabeçalho somente recolhem/expandem; painéis recebem símbolos vetoriais próprios. Reposicionamento continua por arrasto.


## 0.4 — Project layers, desempenho e projetos nativos (revisão interna 1)

- Project layers compartilha grupos, arrasto e olhos com Mission lines; renomear por menu ou duplo clique no nome e excluir com desfazer.
- Nomes de projeto e polígonos e visibilidade individual dos polígonos persistem no formato 6.
- Projetos salvos como `.skylab`, com filtro próprio, extensão e ícone registrados por usuário; JSON antigo continua abrindo. Atualização migra o desinstalador para remover também a nova associação.
- Prévia de geração transporta somente o polígono ativo, atualiza apenas a camada temporária e evita uma validação topológica repetida.
- Painéis acompanham maximização/restauração proporcionalmente, sem acumular deslocamento; referência do tamanho da janela é salva com o layout.
- Seção Validation removida de Help.


## 0.3 — hierarquia, visibilidade e geração modular (revisão interna 2)

- Botões de exportação geral e de seleção MIS no rodapé de Linhas da missão.
- Grupos aninhados com arrasto direto, reordenação, retorno à raiz e renomeação; proveniência e vértices preservados.
- Olhos para grupos, arquivos e linhas; visibilidade herdada, salva em JSON formato 5, com desfazer. Geometria oculta não participa da seleção no mapa nem do corte manual.
- Expansão das camadas deixa de reconstruir a árvore durante o clique.
- Geração em painel modular flutuante junto ao polígono, com campos de numeração abertos e dimensões verificadas em português.
- Cursores próprios para cada ferramenta; prévia violeta ao criar outro conjunto sobre linhas existentes.
- Validação de grupos recusa ciclos, pais inválidos e profundidade excessiva; arquivos antigos continuam legíveis.


## 0.3 — seleção, blocos e acabamento (revisão interna 1)

- Caneta com ícone de pena e atalho P configurável no painel F1.
- Ctrl alterna linhas individuais; menu contextual renomeia a seleção em sequência com prévia e desfazer.
- Painel de linhas acompanha bloco/camada ativos, com identificação visível e contexto separado da seleção.
- Corrigida a altura da linha selecionada nas camadas; números dos painéis removidos.
- Ajuda e Sobre sempre ao final do menu; Jørgen Nissen incluído e logo azul restaurada no Sobre.
- Fechamento de abas informa operação em andamento ou salvamento incompleto; download nativo detecta conclusão imediata.


## 0.2 — polígonos e revisão do código (revisão interna 3)

- Geração e prévia na barra contextual do polígono selecionado; comandos de edição em faixa superior.
- Caneta para polígonos com alças Bézier; inserção de âncoras com Shift, edição por arrasto e histórico. JSON formato 4 preserva as curvas; formatos anteriores continuam legíveis.
- Importação na área vazia e no menu de contexto; camadas permanecem dedicadas aos dados.
- Coordenadas e atribuição XYZ na borda inferior do mapa, sem sobrepor zoom.
- Três revisões independentes cobriram Rust, interface e integração Windows. Removidos caminho legado do instalador, comandos sobrescritos, traduções e CSS sem uso; eliminadas validações repetidas.
- Corrigidos exclusão após seleção por retângulo, redesenho da prévia XYZ e exportação de camadas com curvas; KML inclui polígonos e furos.

## 0.2 — painéis e integração com arquivos (revisão interna 2)

- Painéis móveis e redimensionáveis com encaixe lateral/inferior, posições flutuantes internas, minimizar, ocultar e restaurar layout persistente.
- Ferramenta mão e navegação temporária com Espaço, devolvendo seleção/corte ao soltar.
- Tela inicial com logo transparente e até 12 arquivos recentes, reabertos diretamente do disco.
- Exportação da seleção em MIS preserva todos os vértices e valida nomes.
- Associação MIS por usuário e encaminhamento para a instância aberta; migração do desinstalador ao atualizar da 0.1, com remoção dos registros próprios.
- Logo inversa no About, contribuição duplicada removida e rodapé de testes em todas as telas.
- JSON de projeto continua no formato 3, mantendo compatibilidade com os formatos anteriores.

## 0.1 — liberação do executável na atualização (revisão interna 6)

- Atualização interna aguarda até cinco segundos pela liberação do executável após o processo encerrar. Evita falha intermitente de acesso no Windows, sem encerrar processos à força nem alterar arquivos ainda bloqueados.
- Mantém as funções de camadas, arrasto nos rótulos e troca direta do mapa da revisão 5.


## 0.1 — camadas e controles de edição (revisão interna 5)

- Numeração de projetos sem acumular os já fechados; ícones distintos de atualização e salvamento automático.
- Gerenciamento de camadas por arquivo MIS/MISSION, KML ou KMZ, com árvore de polígonos e linhas.
- Exportação individual por camada em MIS/KML/JSON, preservando geometria e identificação de origem após cortes e divisões.
- KMZ lido sem extrair arquivos. JSON de projeto formato 3; formatos anteriores permanecem legíveis.
- Troca direta do mapa de fundo com enquadramento preservado e opção separada para desligar a imagem.
- Arrasto horizontal nos rótulos numéricos, com prévia dinâmica, sem painel flutuante.
- Hover restrito à imagem do ícone e espaçamentos padronizados entre título, ferramentas e indicadores.


## 0.1 — início, atalhos e recuperação (revisão interna 4)

- Tela de entrada sem projeto ou aba automática; criar, abrir e arrastar arquivo.
- Transição aguarda imagens, fontes e quadros pintados antes de remover a cobertura nativa.
- Créditos do Delphi, colaboradores e Rafael Dantas no Sobre.
- Seleção com seta, corte com tesoura e manutenção dos trechos cortados selecionados.
- Atalhos configuráveis em Ajuda/F1, com persistência local e verificação de conflitos.
- Recuperação local a cada 90 segundos, preservada após falha e limpa no encerramento normal.
- Conexão Google Satellite/Hybrid XYZ conforme configuração do QuickMapServices/QGIS, sem chave de API.


## 0.1 — edição e atualização interna (revisão interna 3)

- Botão de edição: seleção por clique/retângulo, corte por traço e exclusão da seleção.
- Corte em Rust preserva vértices e sentido, com nomes distintos e desfazer por operação.
- Atualização pelo GitHub verifica SHA256 e identidade, aplica após encerrar e reinicia sem outro instalador.
- Restaura até três abas, inclusive projetos não salvos, seleção, enquadramento e campos. Histórico de desfazer reinicia após atualizar.
- Falha na substituição recupera o executável anterior; preserva projetos pessoais, atalhos e desinstalador.
- Instalações anteriores exigem uma instalação manual desta revisão para receber o novo atualizador.

## 0.1 — acabamento da interface (revisão interna 2)

- Primeiro número e incremento mantêm edição numérica e deixam de abrir sliders.
- Ícones do cabeçalho sem borda, com indicação de hover e foco por teclado.
- Menu da logo alinhado à margem esquerda e ao cabeçalho, com a mesma cor azul.
- Sobre centralizado, com logos SkyLab/SkyTEM, SkyTEM Survey Laboratory e suporte +45 29 84 36 90; retirada a consulta de atualização desse diálogo.
- Adicionar malha preservado por solicitação do usuário, com explicação no hover e no foco por teclado.
- Entrada suave da janela e transição da logo para a interface após confirmação de carregamento pelo adaptador nativo.

## 0.1 — atualizador e distribuição

- Controle de revisão interna para reconhecer correções sem mudar a versão 0.1.
- Publicação do instalador com SHA256 pelo GitHub Actions ao atualizar BUILD_REVISION em main.
- Instalações anteriores a esse controle precisam de uma instalação manual inicial.

## 0.1 — correção da abertura (sem avanço de versão)

- Corrigida exceção `Intervalo solicitado ultrapassa o fim da matriz` na conversão da logo do splash pelo .NET Framework. Os quadros PNG do ICO são renderizados pelo identificador nativo do Windows.
- Teste da criação das janelas do aplicativo e do instalador, com a logo de 128 px visível, incluído na compilação obrigatória.
- Falhas genéricas de abertura não são mais apresentadas como falta do WebView2 sem confirmação.

## 0.1 — abas e prévias (sem avanço de versão)

- Cabeçalho compacto com menu na logo, efeito de hover e controles de atualização e idioma.
- Até três projetos com geometria, seleção, enquadramento, parâmetros e histórico independentes; aviso antes de fechar alterações sem salvar.
- Sliders em hover e prévia temporária para geração, corte e divisão. Respostas antigas são descartadas, com uma operação de prévia por vez.
- Numeração em português alinhada; logo exibida durante a abertura da janela.
- Consulta manual de Releases do GitHub e fluxo de publicação do instalador. Revisões com a mesma versão não geram aviso de versão posterior.

## 0.1 — revisão institucional (sem avanço de versão)

- Versão atual mantida em 0.1 por orientação expressa do usuário; próximos avanços exigem pergunta e aprovação.
- SVG branco fornecido pelo usuário substitui a marca raster na interface e no ícone.
- Aplicativo e instalador sem slogans; layout direto, com barras de rolagem azuis.
- Janela Windows própria em WebView2, sem a moldura cinza do navegador, com botões nativos integrados ao azul.
- Instalador com execução em segundo plano, porcentagem e etapas informadas pela operação real.
- Botão Checar atualização com fonte HTTPS ou arquivo configurável; ausência de fonte informada explicitamente.


## Marcação preliminar 0.2 — preservada no histórico, sem aprovação corrente

- Logo SkyTEM original recuperada do About do SkyLab, sem executar o programa antigo; identidade azul/cinza aplicada ao aplicativo e ao instalador.
- Tela Sobre, ícone Windows e informações de versão incorporados nos executáveis.
- Instalador em Rust com interface moderna local, reaproveitando os estilos do planejador.
- Instalação por usuário, atalho no menu Iniciar, atalho opcional na área de trabalho e registro em Aplicativos instalados do Windows.
- Atualização/reparo, cópia anterior do executável e desinstalador com remoção posterior do próprio arquivo.
- Modo de teste isolado para verificar o ciclo sem alterar atalhos pessoais ou registro do Windows.
- Abertura em janela própria do Edge quando disponível; opção de encerrar o aplicativo na tela Sobre.
- Repositório Git local, tags e bundle para migração futura.

## 0.1.0 — 27/10/2026

- Primeira versão de planejamento 2D em Rust, com importação KML/MIS, geração, recorte, edição, corte por distância, renomeação e exportação MIS/KML.
- Projetos JSON, desfazer/refazer, prévia responsiva e validação com o acervo local.

## 0.1 — revisão de operação (sem avanço autorizado)

- Confirmação de CRS em toda abertura: declarado em JSON, sugestão geográfica em KML, confirmação obrigatória em MIS sem metadados.
- Catálogo PROJ do QGIS instalado, 13.571 definições para busca e 6.920 sistemas atuais de trabalho em metros/eixos leste-norte; reprojeção real, sem aproximação silenciosa quando faltam grades.
- Shift para seleção múltipla, destaque sincronizado mapa/lista e menu de ferramentas pelo botão direito.
- Soma de todas as linhas e soma/quantidade selecionadas no fim do painel.
- Histórico otimizado com 30 estados, Ctrl+Z, Ctrl+Y e restauração da seleção/enquadramento.
- English inicial, Português (Brasil) e Dansk; tutorial de sete etapas na primeira utilização e repetição em Ajuda.
- Gravação pelo diálogo nativo, sem painel de downloads ou aviso de saída de página; removido Encerrar do Sobre.
- Satélite opcional via Google Maps JavaScript API, condicionado à chave/conta da empresa. Configuração e falhas de conexão são tratadas; imagem real permanece sem validação por falta da chave.
- Projetos com autoridades CRS usam formato de arquivo 2, mantendo a versão do produto 0.1 e a leitura dos projetos antigos.

## 0.1 — revisão de seleção, atalhos, cabeçalho e marca

- Versão mantida em 0.1 por confirmação expressa do usuário.
- Corrigida referência inválida que interrompia o Ctrl+Z; regressão de excluir/restaurar e onze etapas de histórico cobertas por testes do manipulador real.
- Shift seleciona todas as linhas do intervalo; espaço vazio limpa a seleção; mapa/lista permanecem sincronizados.
- Adicionar arquivo substitui o rótulo quando há projeto aberto. A câmera enquadra a geometria importada, incluindo adição a projetos maiores.
- Comandos de projeto à esquerda, consulta de atualização por ícone, idiomas por bandeiras, Sobre no nome SkyLab e exemplo no final de Ajuda.
- Ícone aprovado aplicado no programa, instalador, janela nativa e executáveis.
- Pesquisa de opções oficiais Google documentada, sem acesso a endpoints internos do site e sem criação de cobranças.

## 0.1 — revisão de código em três frentes

- Removidos implementações substituídas, estados e seletores sem uso, rota JPEG legada, mensagens nativas sem efeito e comando diagnóstico sem chamadas.
- Otimizados validação/geração Rust, inspeção KML, cache PROJ, respostas estáticas, cálculo de comprimentos/limites e formatação/desenho da interface.
- Instalador usa cópia direta do executável, sem buffer integral de aproximadamente 30 MB. Catálogo redundante deixa de ser escrito no cache.
- Testes integrados, Clippy, compilação nativa e ciclo isolado do instalador passaram. Medições e limites em AUDITORIA-OTIMIZACAO-0.1.md.
