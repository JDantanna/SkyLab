# SkyLab — 0.2

Primeira versão utilizável, local, do fluxo de missões do SkyLab para helicópteros. O cálculo e a validação são feitos em Rust; a interface usa uma janela Windows própria e opera localmente. Não utiliza nem executa o programa antigo.

## Layout e arquivos

Os painéis de camadas, malha, revisão, seleção, corte por distância e ações em lote podem ser redimensionados, minimizados, ocultados e movidos entre as laterais, a parte inferior e posições flutuantes dentro da janela SkyLab. Arraste o título ou use o seletor de encaixe; Ajuda > Painéis permite reabrir painéis e restaurar o layout. O layout é lembrado neste computador.

Durante a edição, segure Espaço para mover o mapa e solte para voltar à ferramenta anterior. A mão também está disponível na barra. Botão direito > Exportar seleção · MIS exporta só as linhas selecionadas, preservando todos os vértices.

A tela inicial mostra até 12 arquivos recentes e reabre seu conteúdo atual no disco; apenas os caminhos são guardados na lista. O instalador registra `.mis` e encaminha os arquivos para a instância aberta. Se o Windows já tiver outro aplicativo explicitamente escolhido, selecione SkyLab uma vez em Abrir com > Sempre. O programa respeita essa escolha protegida do Windows.

Esta é uma versão de testes; o rodapé de todas as telas informa o contato Rafael Dantas, rda@skytem.com.

## Camadas e edição

MIS/MISSION, KML e KMZ ficam em grupos separados no painel de camadas. Expanda um arquivo para ver seus polígonos e linhas; botão direito exporta somente esse grupo. Salvar projeto mantém as camadas em JSON formato 3, com leitura dos formatos anteriores. A versão do programa é 0.2.

Arraste os rótulos dos parâmetros para ajustar valores com prévia em tempo real, sem painel flutuante. Clique em Satellite para trocar a conexão diretamente, mantendo o enquadramento. Consulte GUIA-OPERACAO.md.

## Tela inicial, atalhos e recuperação

O programa inicia sem abas. Crie um projeto, abra/importa um arquivo ou arraste-o para a tela inicial. F1 abre os atalhos configuráveis; os padrões são Ctrl+N, Ctrl+O, Ctrl+S, Delete, V e Q. A tesoura mantém os trechos cortados selecionados.

Uma cópia local de recuperação é gravada a cada 90 segundos, indicada ao lado da atualização. Após uma falha, a próxima abertura oferece restaurá-la. Fechar normalmente limpa esse cache. Consulte GUIA-OPERACAO.md.

A opção Google Satellite XYZ utiliza a mesma conexão do QuickMapServices/QGIS, sem chave nesse endpoint. Seu funcionamento foi conferido com projeto sintético; disponibilidade e condições são do fornecedor. A conexão Google Maps Platform com chave continua disponível.

## Instalar e abrir

Obtenha **SkyTEM-SkyLab-Setup-0.2-r2.exe** na [Release 0.2](https://github.com/JDantanna/SkyLab/releases/tag/v0.2-r2) e execute o instalador. A instalação usa uma interface azul/cinza com a logo SkyTEM original, instala somente para o usuário atual, cria entrada no menu Iniciar e oferece atalho na área de trabalho. O destino é `%LOCALAPPDATA%\Programs\SkyTEM\SkyLab`. Não exige acesso de administrador e não instala um serviço permanente. A operação é offline quando os pré-requisitos descritos abaixo estão presentes.

Abra **SkyLab** pelo menu Iniciar ou pelo atalho. A janela Windows usa WebView2 e controles integrados ao cabeçalho azul, sem a moldura do navegador. A versão exibida é **0.2** (0.2.0 no pacote Rust). Antes de qualquer avanço de versão, será solicitada uma decisão explícita do usuário.

O aplicativo requer Windows 64 bits (x64), .NET Framework 4.6.2 ou posterior Microsoft Edge WebView2 Runtime e Microsoft Visual C++ 2015–2022 x64 Runtime. O runtime já está disponível neste computador; em outras máquinas, a TI deve instalá-lo se necessário. Consulte JANELA-WINDOWS.md.

O aplicativo atende normalmente em **http://127.0.0.1:8787**. Se a versão portável anterior estiver usando a porta, encerre-a antes de abrir esta versão ou inicie o executável com outra porta: `SkyLab-Rust.exe --port 8780`. `--no-open` inicia sem abrir a interface. Uma segunda abertura reconhece a instância desta versão quando já está ativa.

Para atualizar, use o símbolo de atualização e confirme **Update and restart**: o aplicativo baixa um pacote verificado e reinicia, recuperando até três abas. Instalações da revisão 2 ou anteriores precisam deste instalador uma vez para ativar o novo mecanismo. Para reparar, feche o SkyLab e execute o instalador. Para remover, use **Aplicativos instalados** do Windows ou `Uninstall.exe` na pasta do programa. Arquivos pessoais e projetos não são removidos. Somente arquivos reconhecidos da instalação são substituídos ou apagados. O instalador não é MSI e instala por usuário; implantação administrativa por máquina ainda não foi implementada.

Os executáveis têm ícone e informações de versão, mas **não possuem assinatura digital corporativa**. A distribuição para outros computadores deve incluir a validação da empresa e, quando disponível, sua assinatura de código.

Use **Abrir exemplo** para experimentar um bloco fictício com um furo. `interface-0.1-institucional.png` registra a interface com a marca SkyTEM; `atualizacao-0.1.png` mostra a consulta de atualização. As capturas contêm somente a demonstração fictícia.

## Fluxo do operador

Consulte **GUIA-OPERACAO.md** para importação com confirmação de CRS, seleção múltipla, ferramentas do botão direito, total de comprimento, histórico de 30 etapas, idiomas, tutorial e camada de satélite opcional. English é o idioma inicial; a escolha de idioma fica salva neste computador. Salvar/exportar usa o diálogo de arquivos do Windows.

## Convenções geométricas

- Coordenadas internas: metros de grade em um CRS projetado leste/norte explícito. WGS84/UTM mantém suas fórmulas verificadas; outras autoridades e transformações de datum usam PROJ offline, sem aproximação silenciosa. Grades necessárias devem estar disponíveis. Consulte GUIA-OPERACAO.md.
- Azimute: graus no sentido horário a partir do **norte de grade**; 0° norte, 90° leste, 180° sul. Não é azimute verdadeiro nem magnético. Distâncias são de grade, sem correção terreno/escala para distância no solo.
- Origem da malha: menor coordenada projetada do contorno na direção perpendicular às linhas. “Deslocamento inicial” soma uma fase a essa origem; deve ser ≥ 0 e menor que o espaçamento. Não equivale ao centro X/Y do SkyLab antigo e não promete reproduzir sua malha ponto a ponto.
- Polígonos externos formam uma **união de áreas de levantamento**; anéis internos são furos. Polígonos separados de “no flight zones” **não são subtraídos**. Prepare essas exclusões como furos em um SIG antes da importação; não as adicione como blocos.
- Polígonos côncavos e furos podem dividir uma linha em partes; a geração numera cada parte separadamente, sem unir as lacunas. O comprimento mínimo é aplicado a cada parte. Inverter o sentido afeta também a referência dos cortes por distância.
- Nomes MIS: ASCII sem espaços, até 64 caracteres, não exclusivamente numéricos, únicos. Na importação KML, recupera `SimpleData ID` quando não há `<name>`; IDs numéricos recebem prefixo L, nomes incompatíveis ganham sequência e repetições ganham sufixo. Toda adaptação aparece nas notas. Na importação MIS, inconsistências são recusadas.
- MIS: nome em uma linha, seguido por **todos os vértices** em Leste Norte; ponto decimal, 2 casas decimais, CRLF, sem cabeçalho. Exportação confere novamente a geometria após arredondar. O nome do arquivo inclui o CRS escolhido, mas o conteúdo MIS não carrega projeção.
- KML de saída contém as linhas, com longitude/latitude WGS84 e altitude zero. O projeto JSON preserva os blocos. Altitudes e estilos de entrada são descartados.

## Limitações e validação operacional

Esta versão **não está homologada para navegação em voo**. Antes de uso operacional, a equipe deve conferir datum, zona, coordenadas e nomes no sistema de bordo, verificar se aceita nomes com sufixos e polilinhas, e validar a missão por seu procedimento operacional. Nenhum teste foi realizado no SkyMap ou equipamento de bordo.

Planejamento 2D: não calcula terreno, obstáculos, altura, curvas, raio de giro, entradas/saídas, autonomia, zonas aeronáuticas nem sequenciamento operacional. A imagem de satélite é opcional e requer configuração Google; não há importação SHP/KMZ, desenho de blocos, edição por arrastar vértices ou vínculo automático com o sistema de bordo. KML precisa de coordenadas geográficas, mesmo quando o arquivo tem “UTM” no nome. KMZ deve ser descompactado externamente.

Limites deliberados: arquivos de entrada até 8 MB, requisições até 10 MB, no modo WGS84/UTM por zona numérica, zona ordinária de 6° e latitudes −80° a 84° (outros sistemas via catálogo PROJ), até 2.000 vértices por anel/5.000 no conjunto de polígonos, 20.000 linhas/200.000 vértices de linhas. Polígonos precisam estar fechados, sem auto-interseções; furos precisam ficar estritamente dentro e não se tocar. Anéis muito detalhados devem ser tratados externamente sem perder os requisitos do levantamento.

Os arquivos reais foram lidos somente localmente; não estão incluídos neste pacote. O aplicativo não envia arquivos de missão, não grava sobre os originais e atende somente em 127.0.0.1. Os arquivos carregados ficam na memória da aplicação até você salvar o projeto ou fechar. A camada Google opcional transmite a área visualizada apenas após ativação. A janela da aplicação não registra coordenadas.

## Implementação e testes

`src/projection.rs`: catálogo e transformação PROJ offline. `src/lib.rs`: projeção WGS84/UTM, leitura/escrita MIS/KML, validação topológica, geração, recorte e corte por distância. `src/editing.rs`: corte por traço preservando vértices. `src/self_update.rs`: atualização interna e recuperação da sessão. `src/main.rs`: servidor local e API de planejamento. `src/installation.rs` e `src/bin/setup.rs`: instalação, atualização e desinstalação. `web/` e `installer/`: interfaces incorporadas nos executáveis. `tests/`: testes independentes do acervo confidencial. `vendor/`: dependências e licenças para compilação offline; `Cargo.lock` fixa as versões. `native/Host.cs` hospeda a janela Windows. `build.rs` compila esse adaptador e incorpora ícone/metadados.

Com Rust instalado, dentro desta pasta:

```text
cargo test --offline
cargo clippy --all-targets --offline -- -D warnings
cargo build --release --offline
```

O executável recompilado fica em `target/release/skylab-rust.exe`. Para construir também o instalador, execute `scripts/build.ps1`: primeiro ele compila o aplicativo, prepara o payload e depois gera `dist/SkyTEM-SkyLab-Setup.exe`. Requer Rust, Node.js 20+ para os testes de interação, Windows SDK com `rc.exe` e o compilador .NET Framework. O modo `--test-root PASTA --no-open` permite testar o instalador com arquivos/atalhos isolados dentro de PASTA/SkyLab, sem modificar o registro do Windows ou atalhos pessoais. O desinstalador de teste exige também `--uninstall --test-root PASTA`.

WGS84/UTM usa séries de Transversa de Mercator de Snyder, restritas à zona; os demais sistemas usam PROJ offline com validação de grades. Os testes incluem os vetores publicados pelo [PROJ](https://proj.org/en/stable/operations/projections/utm.html), ida/volta em várias zonas, hemisférios e latitudes, e comparação geométrica com os exemplos locais. Isso não substitui homologação geodésica nem validação em voo.

Consulte **VALIDACAO.md** para resultados e decisões pendentes, **IDENTIDADE.md** para a origem da marca e **REPOSITORIO.md** para histórico, continuidade no espaço de trabalho e migração para o servidor da empresa.

O símbolo de atualização consulta as Releases de JDantanna/SkyLab. Consulte **ATUALIZACOES.md** para publicar o instalador ou migrar para uma fonte corporativa. A janela usa o SVG branco fornecido pelo usuário; slogans foram removidos.



As licenças das bibliotecas distribuídas são instaladas como `THIRD-PARTY-NOTICES.txt`. A camada PROJ não requer QGIS instalado. A imagem `tutorial-0.1.png` mostra a revisão atual da interface com o tutorial.

A revisão de layout aplica a nova marca SkyLab, organiza os comandos à esquerda, abre Sobre pelo nome, usa bandeiras para idiomas, move o exemplo para Ajuda e corrige o Ctrl+Z. Shift seleciona intervalos completos. Consulte GOOGLE-MAPS-CUSTOS.md para a pesquisa solicitada sobre gratuidade.

A revisão otimizada mantém a versão 0.1. Consulte AUDITORIA-OTIMIZACAO-0.1.md para código removido, medições e validação.
