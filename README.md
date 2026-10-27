# SkyLab — 0.1

Primeira versão utilizável, local, do fluxo de missões do SkyLab para helicópteros. O cálculo e a validação são feitos em Rust; a interface usa uma janela Windows própria e opera localmente. Não utiliza nem executa o programa antigo.

## Instalar e abrir

Execute **SkyTEM-SkyLab-Setup-0.1.exe**, entregue junto ao pacote. A instalação usa uma interface azul/cinza com a logo SkyTEM original, instala somente para o usuário atual, cria entrada no menu Iniciar e oferece atalho na área de trabalho. O destino é `%LOCALAPPDATA%\Programs\SkyTEM\SkyLab`. Não exige acesso de administrador e não instala um serviço permanente. A operação é offline quando os pré-requisitos descritos abaixo estão presentes.

Abra **SkyLab** pelo menu Iniciar ou pelo atalho. A janela Windows usa WebView2 e controles integrados ao cabeçalho azul, sem a moldura do navegador. A versão exibida é **0.1** (0.1.0 no pacote Rust). Antes de qualquer avanço de versão, será solicitada uma decisão explícita do usuário.

O aplicativo requer Windows 64 bits (x64), .NET Framework 4.6.2 ou posterior e Microsoft Edge WebView2 Runtime. O runtime já está disponível neste computador; em outras máquinas, a TI deve instalá-lo se necessário. Consulte JANELA-WINDOWS.md.

O aplicativo atende normalmente em **http://127.0.0.1:8787**. Se a versão portável anterior estiver usando a porta, encerre-a antes de abrir esta versão ou inicie o executável com outra porta: `SkyLab-Rust.exe --port 8780`. `--no-open` inicia sem abrir a interface. Uma segunda abertura reconhece a instância desta versão quando já está ativa.

Para atualizar/reparar, feche o SkyLab e execute o novo instalador. Para remover, use **Aplicativos instalados** do Windows ou `Uninstall.exe` na pasta do programa. Arquivos pessoais e projetos não são removidos. Somente arquivos reconhecidos da instalação são substituídos ou apagados. O instalador não é MSI e instala por usuário; implantação administrativa por máquina ainda não foi implementada.

Os executáveis têm ícone e informações de versão, mas **não possuem assinatura digital corporativa**. A distribuição para outros computadores deve incluir a validação da empresa e, quando disponível, sua assinatura de código.

Use **Abrir exemplo** para experimentar um bloco fictício com um furo. `interface-0.1-institucional.png` registra a interface com a marca SkyTEM; `atualizacao-0.1.png` mostra a consulta de atualização. As capturas contêm somente a demonstração fictícia.

## Fluxo do operador

1. Selecione zona e hemisfério UTM **antes** de importar. O datum desta versão é **WGS84**. KML usa longitude/latitude WGS84; MIS não informa datum ou projeção. Não conclua a projeção apenas pelo nome do arquivo.
2. Importe um KML com **Polygon** para definir o bloco. KML com **LineString** e MIS entram como linhas existentes; não são convertidos em polígonos. “Adicionar ao projeto atual” permite reunir bloco e linhas. Desmarcar substitui a missão. Um projeto JSON sempre abre como projeto completo. Para trocar a projeção, abra **Novo projeto**.
3. Ajuste espaçamento, azimute e comprimento mínimo. Gere a malha; as linhas já saem recortadas pelo bloco. Em “Numeração e sentido”, ajuste prefixo, primeiro número, incremento e alternância. Para acrescentar linhas de controle, marque **Adicionar malha**, use outro prefixo/numeração e uma orientação apropriada.
4. Clique na linha na prévia ou na lista. Renomeie, edite os vértices Leste/Norte, inverta o sentido, mantenha um intervalo de distância ou divida a linha. O primeiro vértice e a seta indicam o sentido. O recorte pelo bloco pode produzir várias partes; cada parte ganha nome próprio. Trechos fora do bloco são removidos.
5. Use a busca e confira os comprimentos. Arraste a prévia para mover, use a roda ou +/− para zoom e **Enquadrar** para retornar. Desfazer/refazer mantém até 30 alterações (Ctrl+Z/Ctrl+Y fora dos campos de texto).
6. **Salvar projeto** baixa JSON com projeção, blocos, furos, linhas e notas. Salve regularmente: não há gravação automática. Histórico e valores dos controles de geração não são persistidos. **Exportar missão** baixa MIS para o fluxo SkyMap e KML para conferência. Os downloads seguem a pasta configurada no navegador.

## Convenções geométricas

- Coordenadas internas: metros **de grade UTM**, com zona e hemisfério explícitos. Não há transformação de datum. SIRGAS2000, SAD69 e outros referenciais precisam de conversão externa apropriada antes do uso.
- Azimute: graus no sentido horário a partir do **norte de grade**; 0° norte, 90° leste, 180° sul. Não é azimute verdadeiro nem magnético. Distâncias são de grade, sem correção terreno/escala para distância no solo.
- Origem da malha: menor coordenada projetada do contorno na direção perpendicular às linhas. “Deslocamento inicial” soma uma fase a essa origem; deve ser ≥ 0 e menor que o espaçamento. Não equivale ao centro X/Y do SkyLab antigo e não promete reproduzir sua malha ponto a ponto.
- Polígonos externos formam uma **união de áreas de levantamento**; anéis internos são furos. Polígonos separados de “no flight zones” **não são subtraídos**. Prepare essas exclusões como furos em um SIG antes da importação; não as adicione como blocos.
- Polígonos côncavos e furos podem dividir uma linha em partes; a geração numera cada parte separadamente, sem unir as lacunas. O comprimento mínimo é aplicado a cada parte. Inverter o sentido afeta também a referência dos cortes por distância.
- Nomes MIS: ASCII sem espaços, até 64 caracteres, não exclusivamente numéricos, únicos. Na importação KML, recupera `SimpleData ID` quando não há `<name>`; IDs numéricos recebem prefixo L, nomes incompatíveis ganham sequência e repetições ganham sufixo. Toda adaptação aparece nas notas. Na importação MIS, inconsistências são recusadas.
- MIS: nome em uma linha, seguido por **todos os vértices** em Leste Norte; ponto decimal, 2 casas decimais, CRLF, sem cabeçalho. Exportação confere novamente a geometria após arredondar. O nome do download inclui WGS84 e UTM, mas o conteúdo MIS não carrega projeção.
- KML de saída contém as linhas, com longitude/latitude WGS84 e altitude zero. O projeto JSON preserva os blocos. Altitudes e estilos de entrada são descartados.

## Limitações e validação operacional

Esta versão **não está homologada para navegação em voo**. Antes de uso operacional, a equipe deve conferir datum, zona, coordenadas e nomes no navegador de bordo, verificar se aceita nomes com sufixos e polilinhas, e validar a missão por seu procedimento operacional. Nenhum teste foi realizado no SkyMap ou equipamento de bordo.

Planejamento 2D: não calcula terreno, obstáculos, altura, curvas, raio de giro, entradas/saídas, autonomia, zonas aeronáuticas nem sequenciamento operacional. Não há mapa de fundo, importação SHP/KMZ, desenho de blocos, edição por arrastar vértices, nem vínculo automático com o navegador de bordo. KML precisa de coordenadas geográficas, mesmo quando o arquivo tem “UTM” no nome. KMZ deve ser descompactado externamente.

Limites deliberados: arquivos de entrada até 8 MB, requisições até 10 MB, zona UTM ordinária de 6° (não além dela, nem zonas especiais Noruega/Svalbard), latitudes −80° a 84°, até 2.000 vértices por anel/5.000 no conjunto de polígonos, 20.000 linhas/200.000 vértices de linhas. Polígonos precisam estar fechados, sem auto-interseções; furos precisam ficar estritamente dentro e não se tocar. Anéis muito detalhados devem ser tratados externamente sem perder os requisitos do levantamento.

Os arquivos reais foram lidos somente localmente; não estão incluídos neste pacote. O aplicativo não faz upload, não usa mapas externos, não grava sobre os originais e atende somente em 127.0.0.1. Os arquivos carregados ficam na memória da página até você baixar o projeto ou fechar. A janela da aplicação não registra coordenadas.

## Implementação e testes

`src/lib.rs`: projeção WGS84/UTM, leitura/escrita MIS/KML, validação topológica, geração, recorte e corte por distância. `src/main.rs`: servidor local e API de planejamento sem gravação de arquivos. `src/installation.rs` e `src/bin/setup.rs`: instalação, atualização e desinstalação. `web/` e `installer/`: interfaces incorporadas nos executáveis. `tests/`: testes independentes do acervo confidencial. `vendor/`: dependências e licenças para compilação offline; `Cargo.lock` fixa as versões. `native/Host.cs` hospeda a janela Windows. `build.rs` compila esse adaptador e incorpora ícone/metadados.

Com Rust instalado, dentro desta pasta:

```text
cargo test --offline
cargo clippy --all-targets --offline -- -D warnings
cargo build --release --offline
```

O executável recompilado fica em `target/release/skylab-rust.exe`. Para construir também o instalador, execute `scripts/build.ps1`: primeiro ele compila o aplicativo, prepara o payload e depois gera `dist/SkyTEM-SkyLab-Setup.exe`. Requer Rust, Windows SDK com `rc.exe` e o compilador .NET Framework. O modo `--test-root PASTA --no-open` permite testar o instalador com arquivos/atalhos isolados dentro de PASTA/SkyLab, sem modificar o registro do Windows ou atalhos pessoais. O desinstalador de teste exige também `--uninstall --test-root PASTA`.

A transformação usa séries de Transversa de Mercator de Snyder, restritas à zona; testes incluem os vetores publicados pelo [PROJ](https://proj.org/en/stable/operations/projections/utm.html), ida/volta em várias zonas, hemisférios e latitudes, e comparação geométrica com os exemplos locais. Isso não substitui homologação geodésica nem validação em voo.

Consulte **VALIDACAO.md** para resultados e decisões pendentes, **IDENTIDADE.md** para a origem da marca e **REPOSITORIO.md** para histórico, continuidade no espaço de trabalho e migração para o servidor da empresa.

O botão **Checar atualização** consulta uma fonte configurável. Enquanto o servidor/arquivo da empresa não for definido, informa que a fonte não está configurada. Consulte **ATUALIZACOES.md**. A janela usa o SVG branco fornecido pelo usuário; slogans foram removidos.


