# SkyLab 0.3 — revisão de operação

## Tela inicial e atalhos

O programa abre sem projetos e sem abas. Use Novo projeto (Ctrl+N), Abrir (Ctrl+O) ou arraste um arquivo para a tela inicial. Fechar a última aba retorna a essa tela. O próximo projeto volta ao número 1; números adicionais identificam as abas abertas, sem seguir o total histórico de projetos criados.

Ajuda → Atalhos de teclado, ou F1, permite mudar os atalhos. A ordem e os valores iniciais são: Novo projeto Ctrl+N; Abrir/importar Ctrl+O; Salvar Ctrl+S; Excluir Delete; Selecionar V; Cortar Q. Clique no campo e pressione a combinação; conflitos e combinações reservadas são recusados. Preferências ficam no computador. Ctrl+Z/Ctrl+Y continuam disponíveis. Digitar em campos não aciona atalhos simples como V, Q ou Delete.

## Recuperação automática

A cada 90 segundos, uma cópia local das abas, projetos, seleção, enquadramento e parâmetros é gravada. O ícone ao lado da atualização gira durante a gravação e seu tooltip indica o último salvamento ou uma falha. O arquivo anterior só é substituído após concluir a gravação da nova cópia. A cópia não substitui Salvar projeto.

Após uma interrupção inesperada, a próxima abertura oferece Recuperar ou Descartar. O histórico de desfazer inicia uma nova sessão. Fechar normalmente limpa o cache de recuperação, preservando os arquivos de projeto salvos. Dados de recuperação não são enviados a serviços externos.

## Abrir e trabalhar

O idioma inicial de um perfil novo é English, independentemente do idioma do Windows. O seletor por bandeira permite English, Português (Brasil) e Dansk; a escolha fica neste computador. Os projetos não mudam de idioma e os nomes das linhas não são traduzidos.

O tutorial aparece no primeiro uso, destaca sete partes da interface e explica cada uma com uma janela e seta. Concluir impede a abertura automática nas próximas sessões. Fazer depois adia para uma próxima sessão. Ajuda → Iniciar tutorial permite refazê-lo. Preferências e conclusão do tutorial ficam no perfil local da janela, não no projeto.

Ao abrir cada arquivo, o diálogo de coordenadas sempre aparece antes de alterar o projeto:

- JSON: lê a projeção declarada no projeto.
- KML: sabe-se que a origem é longitude/latitude WGS84; a localização sugere uma zona UTM. A sugestão não é apresentada como projeção declarada. Áreas que atravessam zonas ou hemisférios recebem aviso.
- MIS: não contém datum, zona ou hemisfério. O operador precisa confirmar esses dados ou escolher o catálogo. Coordenadas numéricas e nome de arquivo não permitem identificar uma zona com segurança.

É possível manter o sistema atual ou adotar o escolhido. Ao adicionar dados em outro sistema, os dados existentes são reprojetados antes da união; nunca se juntam coordenadas de sistemas diferentes apenas trocando o rótulo. Cancelar não altera o projeto. Falhas de transformação também preservam o projeto anterior.

Clique na logo SkyLab para abrir o menu com Novo projeto, Salvar projeto, Exportar, Ajuda e Sobre. A faixa abaixo do cabeçalho contém até três abas independentes. O ícone de duas setas consulta atualizações. Abrir exemplo fica no fim de Ajuda. O botão de arquivo passa de Selecionar arquivo a Adicionar arquivo quando um projeto está aberto; após cada importação, o mapa enquadra a geometria recém-importada.

## Camadas por arquivo

A seção Camadas organiza cada arquivo importado separadamente. Expanda o grupo do projeto e o nome do arquivo para ver polígonos e linhas. Clique em uma linha para selecioná-la no mapa e no painel de revisão. Clique em um polígono para enquadrá-lo. Grupos grandes carregam os itens sob demanda; Mostrar mais revela os seguintes sem construir milhares de elementos inicialmente.

MIS e MISSION são tratados como o mesmo formato textual de linhas. KML e KMZ podem conter polígonos e linhas. KMZ utiliza doc.kml ou um único KML do arquivo compactado; múltiplos KML sem principal são recusados. Não são extraídos arquivos para pastas e NetworkLinks não são baixados. Importações continuam pedindo a confirmação do CRS, individualmente, inclusive ao escolher vários arquivos.

Botão direito no arquivo/camada oferece Enquadrar e Exportar MIS, KML ou JSON somente daquele grupo. MIS fica indisponível para camadas sem linhas. Nomes iguais em arquivos diferentes não são alterados; para um MIS combinado, renumere as linhas ou exporte cada camada separadamente. Cortes, recortes, divisão e renomeação conservam a associação ao arquivo. Linhas geradas aparecem em um grupo próprio.

Salvar projeto grava a estrutura inteira em JSON de formato 3. O número 3 é a versão do arquivo, não a versão do aplicativo, que continua 0.1. Projetos anteriores dos formatos 1 e 2 são legíveis, mas sua origem já mesclada não pode ser reconstruída: aparecem como um grupo existente. Revisões anteriores do programa recusam JSON formato 3, evitando descartar camadas silenciosamente.

## Arrasto nos rótulos

Passe sobre Spacing, Grid azimuth, Initial offset, Minimum length ou os rótulos de corte/divisão por distância. O cursor horizontal indica que o nome pode ser arrastado. Segure o botão esquerdo e mova para a direita para aumentar ou para a esquerda para diminuir. A prévia é atualizada durante o movimento, sem cobrir os controles. Shift refina o ajuste; Ctrl acelera; Escape durante o arrasto restaura o valor inicial. Digitar diretamente no campo continua disponível. Primeiro número e incremento permanecem sem esse arrasto.

## Seleção e edição

Clique seleciona uma linha no mapa e na lista. Shift + clique em outra linha seleciona todo o intervalo entre a primeira e a última, na ordem da lista, inclusive ao selecionar pelo mapa. Clicar no espaço vazio do mapa ou da lista limpa a seleção. Ao selecionar no mapa, a lista revela a linha correspondente, inclusive limpando um filtro que a escondia. O destaque aparece em ambas as áreas.

Botão direito no mapa ou na lista abre ferramentas da seleção: enquadrar, inverter, recortar pelo bloco e excluir. Inverter, recortar e excluir atuam em todas as selecionadas em uma única operação de histórico. Nome, vértices, corte por distância e divisão são operações individuais e ficam indisponíveis com seleção múltipla.

O fim do painel de linhas mostra a soma de todas as linhas e, separadamente, quantidade e comprimento das selecionadas. A soma não depende do filtro de nomes. O mapa não permite selecionar texto ao arrastar.

Ctrl+Z desfaz; Ctrl+Y ou Ctrl+Shift+Z refaz. Há até 30 etapas na sessão, com referências aos estados imutáveis dos projetos para evitar cópias adicionais no histórico. Também são restaurados seleção, enquadramento e identificação do arquivo. Editar depois de desfazer inicia um novo ramo e elimina o refazer anterior. Dentro de um campo de texto, Ctrl+Z mantém o comportamento de edição do próprio campo; o botão Desfazer atua no projeto. O histórico não é gravado no arquivo JSON.

Salvar e exportar usam o diálogo nativo de arquivos. O programa só informa sucesso depois da conclusão da gravação; cancelar mantém o projeto como não salvo. Não há moldura, menus de navegação, aviso de saída de página ou painel de downloads. Sobre contém as marcas SkyLab/SkyTEM, versão e contato de suporte; fechar o programa usa o botão da janela.

## Corte por traço

Clique no lápis junto de Satélite e Enquadrar para abrir as ferramentas. Em **Cut with stroke**, pressione e arraste sobre as linhas para dividi-las onde houver interseção. Os vértices originais, as curvas e o sentido são preservados; os trechos recebem nomes distintos. O corte não apaga nenhum trecho. Os trechos resultantes ficam selecionados, em tonalidades alternadas, preservando também a seleção de linhas não cortadas.

Em **Select**, clique em um trecho ou arraste um retângulo para selecionar vários; Shift acrescenta ao conjunto. A lista acompanha o destaque do mapa. Use **Delete selected lines** ou Delete para excluir. Ctrl+Z desfaz cada corte ou exclusão como uma operação. Desative o lápis para voltar ao arrasto de navegação do mapa. Traços sobrepostos ao longo de uma linha não criam cortes; cruze a linha na posição desejada.

## Atualização interna

Clique no símbolo de atualização e, quando houver publicação posterior, confirme **Update and restart**. O programa verifica e aplica o pacote do GitHub e reinicia. Recupera as três abas, inclusive dados não salvos, seleção, enquadramento e valores dos campos. O histórico de desfazer inicia uma nova sessão após reiniciar. Durante a atualização, aguarde o término.

Revisões 2 ou anteriores precisam executar o instalador desta revisão uma vez para receber o novo mecanismo. Atualizações futuras usam o fluxo interno. Detalhes em ATUALIZACOES.md.

## Catálogo de coordenadas

A distribuição incorpora PROJ 9.6 e o banco fornecido com QGIS 3.40.7 instalado nesta máquina: **13.571 definições** de diferentes autoridades, incluindo EPSG e ESRI. Busca por nome ou código. É uma base fixa dessa distribuição; não significa equivalência a toda versão futura do QGIS, definições personalizadas do usuário ou todos os recursos 3D do QGIS.

O catálogo reúne sistemas projetados, geográficos, verticais, compostos e definições históricas. **6.920 sistemas atuais projetados em metros com eixos leste/norte** são candidatos ao planejamento 2D. Os demais ficam para consulta: comprimentos, espaçamento e azimute da malha exigem um plano métrico leste/norte. A interface normaliza a ordem dos eixos com PROJ. A disponibilidade de uma definição não garante que a transformação tenha todas as grades de datum necessárias.

Rede PROJ desligada explicitamente. Não são permitidas transformações de datum aproximadas (“ballpark”); a melhor operação deve estar disponível. A distribuição inclui o banco e uma grade pequena para o teste NAD83 de Minnesota. Outras grades licenciadas podem ser colocadas em `proj-data` ao lado do executável instalado. A falta de uma grade produz erro e não altera o projeto. O pacote não incorpora os aproximadamente 800 MB de grades que existem no QGIS desta máquina.

SIRGAS 2000 / UTM 23S, ETRS89 / UTM 32N e NAD83 / UTM 15N foram exercitados nos testes. As fórmulas WGS84/UTM originais e suas verificações continuam disponíveis. A biblioteca de geometria, a geração e o recorte permanecem em Rust.

Projetos que armazenam uma autoridade de CRS usam formato JSON de projeto 2, para que o SkyLab antigo recuse esses arquivos em vez de ignorar o novo datum. Isso é a versão do formato de arquivo; a versão do programa continua **0.1**. Projetos antigos WGS84/UTM permanecem legíveis.

## Satélite opcional

O botão Satellite abre as opções da imagem, inclusive quando ela já está ativa, e permite escolher Google Satellite, Google Hybrid ou Google Maps via XYZ, ou a conexão Google Maps Platform com chave.

Com XYZ ativo, escolher outro tipo troca a imagem imediatamente e mantém o enquadramento. Desligar imagem é uma opção separada.

A opção XYZ reproduz a configuração Google Satellite do QuickMapServices instalado no QGIS: imagens visíveis são solicitadas a `https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}`, com zoom até 20. Hybrid usa `lyrs=y`. Não exige uma chave nesse endpoint. O fato de responder sem chave não é uma licença nem uma garantia de disponibilidade gratuita oficial; condições do fornecedor continuam aplicáveis. A tela mantém atribuição e link para os termos. Não há download em massa, pré-carregamento de áreas ou armazenamento offline de imagens pelo SkyLab.

A imagem usa Web Mercator e as linhas são transformadas do CRS do projeto para WGS84 em Rust/PROJ; pan, zoom, seleção e corte usam a mesma câmera. O serviço recebe a área visualizada e o IP, sem envio dos arquivos de missão. Sem ativar a camada, nenhuma imagem é solicitada. O funcionamento vetorial permanece offline.

A opção Google Maps Platform preserva o renderizador oficial, chave Maps JavaScript API e suas configurações. Custos e condições dessa API são próprios da conta configurada. Detalhes e fontes em GOOGLE-MAPS-CUSTOS.md.

## Pré-requisitos e verificação

Windows x64, .NET Framework 4.6.2+, WebView2 Runtime e Microsoft Visual C++ 2015–2022 x64 Runtime. Todos estão presentes nesta máquina. As bibliotecas e licenças PROJ são incluídas; QGIS não precisa estar instalado para usar o SkyLab. Grades adicionais são opcionais e dependem do datum.

Instalador e desinstalador continuam preservando projetos, configurações e arquivos não pertencentes ao produto. Ao usar o instalador manualmente, feche todas as instâncias de SkyLab; a atualização interna encerra e reinicia a própria sessão. As revisões antigas podem estar abertas em outra janela; o programa não as encerra à força.

Consulte GOOGLE-MAPS-CUSTOS.md para as franquias oficiais e limitações de uso gratuito. A marca aplicada no programa, instalador e executáveis é o S original com laboratório e linha aprovado pelo usuário.


## Painéis e navegação durante a edição

Os comandos de gerar linhas ficam na barra contextual do polígono, sem painel próprio. Arraste o título dos demais painéis para reposicioná-los. Solte nas zonas Esquerda, Direita ou Embaixo para encaixar; solte dentro da área de trabalho para manter flutuante. O seletor no título permite o mesmo encaixe pelo teclado. Arraste a borda entre uma lateral e o mapa para mudar sua largura; a alça inferior de cada painel muda sua altura, ou altura/largura quando flutuante. O botão − minimiza, × oculta. Ajuda → Painéis reabre os painéis e restaura o layout. O posicionamento é interno à janela SkyLab e é lembrado neste computador.

A ferramenta Mover (mão) fica junto de Selecionar e Cortar. Segure Espaço para mover temporariamente durante a edição; soltar restaura a ferramenta anterior. O atalho não interfere na digitação em campos. Não modifica geometria nem cria passos no histórico.

## Arquivos recentes e abertura MIS

A tela inicial lista os últimos 12 arquivos abertos pelo seletor nativo, arrasto ou associação do Windows. Clique no nome para reabrir do disco; a projeção é confirmada novamente. Arquivos movidos ou excluídos mostram um aviso e podem ser removidos da lista com ×. A lista guarda apenas caminhos e nomes neste computador, sem copiar o levantamento.

O instalador registra MIS no Windows para este usuário. Duplo clique abre uma aba; com três abas abertas, feche uma para receber o arquivo pendente. Se outro aplicativo já tiver sido escolhido como padrão, use Abrir com → SkyLab → Sempre uma vez. A atualização interna da 0.1 também instala esse registro e migra o desinstalador.

Selecione linhas no mapa ou na lista; botão direito → Exportar seleção · MIS exporta somente elas. Shift mantém a seleção de intervalos. Todos os vértices são preservados; nomes duplicados precisam ser resolvidos ou exportados por camada separada.


## Caneta e polígonos

Clique em Editar geometria e Caneta. Um projeto vazio pede confirmação de projeção e começa numa vista de referência da zona escolhida; posicione o desenho pelos dados e coordenadas da equipe. Clique para criar âncoras retas; clique e arraste para criar alças de curva. Clique no primeiro ponto ou pressione Enter para fechar. Escape cancela. Ctrl+Z/Ctrl+Y durante o traçado retiram/restauram pontos. O contorno só entra no projeto depois da validação geométrica; contornos que cruzam a si mesmos são recusados.

O polígono aparece em Camadas → Polígonos desenhados. Selecione-o no mapa ou nessa árvore: a barra superior mostra espaçamento, azimute, deslocamento, comprimento mínimo e numeração, com prévia em tempo real. Gerar atua somente no polígono selecionado. Adicionar malha mantém as linhas existentes; desmarcado, a geração substitui as linhas atuais, como nas versões anteriores.

Shift + clique no contorno externo selecionado insere uma âncora mantendo o formato da curva. Com Selecionar, arraste pontos ou alças; Alt permite separar as alças. Mover um ponto também move suas alças. Cortar continua sendo a ferramenta de divisão de linhas; a caneta cria polígonos, sem criar linhas de missão avulsas.

Salvar projeto preserva âncoras e alças em JSON formato 4. Formatos anteriores continuam legíveis; versões antigas do aplicativo não abrem o formato novo. Reprojetar lineariza as curvas e deixa uma nota, mantendo o contorno. Exportar a camada em JSON preserva edição; em KML exporta o contorno e seus furos. Os cálculos têm tolerância de 0,1 m na aproximação das curvas.

Clique no botão de importação da área vazia, use Abrir no menu SkyLab ou Adicionar arquivo no botão direito do mapa. O painel de camadas fica dedicado à organização dos dados.

## Seleção e renomeação em lote

O painel Linhas da missão acompanha o bloco ativo indicado acima da lista. Selecione um polígono para trocar de bloco ou o título de uma camada para trabalhar naquela origem. Ctrl + clique alterna linhas individuais, Shift seleciona intervalos e botão direito > Renomear linhas abre a sequência com prévia. P ativa a Caneta; F1 permite trocar o atalho.
