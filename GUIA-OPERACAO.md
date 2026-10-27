# SkyLab 0.1 — revisão de operação

## Abrir e trabalhar

O idioma inicial de um perfil novo é English, independentemente do idioma do Windows. O seletor permite English, Português (Brasil) e Dansk; a escolha fica neste computador. Os projetos não mudam de idioma e os nomes das linhas não são traduzidos.

O tutorial aparece no primeiro uso, destaca sete partes da interface e explica cada uma com uma janela e seta. Concluir impede a abertura automática nas próximas sessões. Fazer depois adia para uma próxima sessão. Ajuda → Iniciar tutorial permite refazê-lo. Preferências e conclusão do tutorial ficam no perfil local da janela, não no projeto.

Ao abrir cada arquivo, o diálogo de coordenadas sempre aparece antes de alterar o projeto:

- JSON: lê a projeção declarada no projeto.
- KML: sabe-se que a origem é longitude/latitude WGS84; a localização sugere uma zona UTM. A sugestão não é apresentada como projeção declarada. Áreas que atravessam zonas ou hemisférios recebem aviso.
- MIS: não contém datum, zona ou hemisfério. O operador precisa confirmar esses dados ou escolher o catálogo. Coordenadas numéricas e nome de arquivo não permitem identificar uma zona com segurança.

É possível manter o sistema atual ou adotar o escolhido. Ao adicionar dados em outro sistema, os dados existentes são reprojetados antes da união; nunca se juntam coordenadas de sistemas diferentes apenas trocando o rótulo. Cancelar não altera o projeto. Falhas de transformação também preservam o projeto anterior.

## Seleção e edição

Clique seleciona uma linha no mapa e na lista. Shift + clique acrescenta ou retira linhas da seleção. Ao selecionar no mapa, a lista revela a linha correspondente, inclusive limpando um filtro que a escondia. O destaque aparece em ambas as áreas.

Botão direito no mapa ou na lista abre ferramentas da seleção: enquadrar, inverter, recortar pelo bloco e excluir. Inverter, recortar e excluir atuam em todas as selecionadas em uma única operação de histórico. Nome, vértices, corte por distância e divisão são operações individuais e ficam indisponíveis com seleção múltipla.

O fim do painel de linhas mostra a soma de todas as linhas e, separadamente, quantidade e comprimento das selecionadas. A soma não depende do filtro de nomes. O mapa não permite selecionar texto ao arrastar.

Ctrl+Z desfaz; Ctrl+Y ou Ctrl+Shift+Z refaz. Há até 30 etapas na sessão, com referências aos estados imutáveis dos projetos para evitar cópias adicionais no histórico. Também são restaurados seleção, enquadramento e identificação do arquivo. Editar depois de desfazer inicia um novo ramo e elimina o refazer anterior. Dentro de um campo de texto, Ctrl+Z mantém o comportamento de edição do próprio campo; o botão Desfazer atua no projeto. O histórico não é gravado no arquivo JSON.

Salvar e exportar usam o diálogo nativo de arquivos. O programa só informa sucesso depois da conclusão da gravação; cancelar mantém o projeto como não salvo. Não há moldura, menus de navegação, aviso de saída de página ou painel de downloads. Sobre contém marca, versão e atualização; fechar o programa usa o botão da janela.

## Catálogo de coordenadas

A distribuição incorpora PROJ 9.6 e o banco fornecido com QGIS 3.40.7 instalado nesta máquina: **13.571 definições** de diferentes autoridades, incluindo EPSG e ESRI. Busca por nome ou código. É uma base fixa dessa distribuição; não significa equivalência a toda versão futura do QGIS, definições personalizadas do usuário ou todos os recursos 3D do QGIS.

O catálogo reúne sistemas projetados, geográficos, verticais, compostos e definições históricas. **6.920 sistemas atuais projetados em metros com eixos leste/norte** são candidatos ao planejamento 2D. Os demais ficam para consulta: comprimentos, espaçamento e azimute da malha exigem um plano métrico leste/norte. A interface normaliza a ordem dos eixos com PROJ. A disponibilidade de uma definição não garante que a transformação tenha todas as grades de datum necessárias.

Rede PROJ desligada explicitamente. Não são permitidas transformações de datum aproximadas (“ballpark”); a melhor operação deve estar disponível. A distribuição inclui o banco e uma grade pequena para o teste NAD83 de Minnesota. Outras grades licenciadas podem ser colocadas em `proj-data` ao lado do executável instalado. A falta de uma grade produz erro e não altera o projeto. O pacote não incorpora os aproximadamente 800 MB de grades que existem no QGIS desta máquina.

SIRGAS 2000 / UTM 23S, ETRS89 / UTM 32N e NAD83 / UTM 15N foram exercitados nos testes. As fórmulas WGS84/UTM originais e suas verificações continuam disponíveis. A biblioteca de geometria, a geração e o recorte permanecem em Rust.

Projetos que armazenam uma autoridade de CRS usam formato JSON de projeto 2, para que o SkyLab antigo recuse esses arquivos em vez de ignorar o novo datum. Isso é a versão do formato de arquivo; a versão do programa continua **0.1**. Projetos antigos WGS84/UTM permanecem legíveis.

## Satélite opcional

O botão Satellite liga/desliga a imagem. A primeira ativação pede uma chave **Google Maps Platform**, com **Maps JavaScript API**, faturamento e restrições adequadas ao aplicativo configurados pela empresa. Não existe chave incluída no código ou no instalador. A chave pode ser usada apenas na sessão ou, mediante seleção do usuário, guardada no perfil local. Não é um cofre de credenciais. A TI deve restringir a chave e as cotas.

A implementação usa o renderizador oficial Google Maps com o tipo satellite, logo, atribuições e links próprios do Google; não usa URLs de tiles não documentadas. A empresa deve cumprir as políticas do Google e disponibilizar seus termos e política de privacidade antes da distribuição dessa integração. Uso pode ser cobrado pelo Google. Referências: [configuração](https://developers.google.com/maps/documentation/javascript/get-api-key), [políticas](https://developers.google.com/maps/documentation/javascript/policies).

Ao habilitar, Google recebe a área visualizada, IP e solicitações normais do serviço. O código do SkyLab não envia arquivos de missão nem chama serviços de geocodificação. Sem ativação, não há solicitações ao Google. A camada vetorial local continua funcionando sem conexão e sem chave.

As linhas são convertidas do CRS do projeto para longitude/latitude WGS84 em Rust/PROJ, com amostragem intermediária dos segmentos a cada 1 km, e desenhadas pelo OverlayView na mesma projeção/pan/zoom da imagem. Isso usa **UTM/CRS**, não UTC (hora). A camada de satélite pode ter deslocamentos ou imagens antigas e não substitui o levantamento. A imagem real não foi validada nesta entrega porque uma chave habilitada da empresa não foi fornecida.

## Pré-requisitos e verificação

Windows x64, .NET Framework 4.6.2+, WebView2 Runtime e Microsoft Visual C++ 2015–2022 x64 Runtime. Todos estão presentes nesta máquina. As bibliotecas e licenças PROJ são incluídas; QGIS não precisa estar instalado para usar o SkyLab. Grades adicionais são opcionais e dependem do datum.

Instalador e desinstalador continuam preservando projetos, configurações e arquivos não pertencentes ao produto. Antes de atualizar, feche todas as instâncias de SkyLab. As revisões antigas podem estar abertas em outra janela; o programa não as encerra à força.
