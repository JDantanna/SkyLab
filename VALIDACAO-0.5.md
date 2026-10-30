# SkyLab 0.5 — cartografia e importação QGIS

Versão 0.5 aprovada explicitamente. Revisão de distribuição 1.

## Edição

- Shift+C alterna o modo depois de carregar geometria. Painéis e subheader dão lugar ao workspace ampliado e a uma barra vertical arrastável; o layout anterior é restaurado ao sair.
- Seleção V, corte Q, caneta P, mão/Espaço, excluir, emenda, aplicar/cancelar, satélite, enquadramento, desfazer/refazer e saída ficam acessíveis na barra.
- Selecionar uma linha mostra pontos e alças. Shift+clique insere um ponto sem alterar a curva; arrastar move pontos/alças. Alt separa tangentes; Delete exclui o ponto ativo sem remover a linha inteira, mantendo pelo menos dois pontos.
- Caneta também converte pontos intermediários em curvas ao arrastar; Alt+clique remove as alças. Desenha linhas abertas; clique+arrasto cria alças, e um extremo existente pode iniciar extensão. Enter aplica, Esc cancela. O desenho pendente tem desfazer/refazer próprio.
- Emenda exige duas linhas e mostra prévia verde da conexão. Somente Enter/aplicar muda o projeto; Esc cancela. Conexões acrescentadas são sinalizadas para inspeção.
- Curvas são calculadas e validadas no Rust, persistidas no formato 7 e achatadas com tolerância de 0,1 m. MIS exporta os vértices resultantes; projetos antigos abrem normalmente. Um arquivo salvo com curvas exige esta versão ou posterior.
- Toda edição aplicada entra no histórico; seleção, camada e coordenadas são preservadas. Nenhuma ferramenta detecta obstáculos automaticamente: o operador faz o desvio e revisa a conexão.

## QGIS

- Projetos .qgs, .qgz e definições .qlr; fontes vetoriais locais GeoPackage, Shapefile, GeoJSON, KML/KMZ. QGZ é lido sem extrair caminhos para o disco.
- Grupos, ordem, visibilidade, nomes de camadas, polígonos, furos, linhas e CRS são importados. O sistema de trabalho é confirmado pelo operador; PROJ faz transformações offline sem aproximação de datum.
- Projetos QGIS normalmente referenciam arquivos externos: eles precisam estar disponíveis nos caminhos registrados. Um GeoPackage avulso com várias tabelas requer um projeto QGIS identificando as tabelas. Shapefile avulso requer .prj; o projeto QGIS também pode declarar o CRS.
- Não são importados estilos, layouts de impressão, plugins, rasters, pontos ou tabelas de atributos. Fontes remotas, ausentes, filtros não suportados e geometrias incompatíveis são informados nas notas. Se nenhuma geometria suportada puder ser lida, a abertura falha com explicação.
- Originais não são modificados; a gravação continua em .skylab. GeoPackage é aberto em modo somente leitura, com limites de tamanho/feições/vértices; todos os dados dos testes são sintéticos.
- São preservados os limites geométricos já usados no programa: até 20.000 linhas e 200.000 vértices de linhas; polígonos seguem a validação existente. Importações que excedem limites são recusadas, sem simplificação automática.

## Evidências

- Testes Rust novos: edição e persistência de curvas, inserção de ponto, emenda orientada por extremos, rejeição de controles inválidos, QGS com DOCTYPE, grupos/visibilidade, furos, CRS, fontes ausentes/remotas, entidades XML, leitura GeoPackage sem modificar o arquivo, QGZ e Shapefile truncado.
- Testes JavaScript de desenho pendente/desfazer/refazer, emenda somente após confirmação e preservação do formato 7; testes anteriores de seleção, corte, atalhos, painéis e recuperação mantidos.
- Verificação interativa: Shift+C ampliou o workspace; seleção mostrou pontos; Shift inseriu um ponto e o arrasto desviou a linha; arrasto com satélite ativo validou a reprojeção. Caneta criou linha curva, levando 12 para 13 linhas. Emenda manteve 13 na prévia, passou a 12 somente ao aplicar e voltou a 13 ao desfazer. Sair restaurou os painéis.
- Importação real pela fila de abertura: QGZ sintético com duas tabelas de GeoPackage externo trouxe uma linha, um polígono, grupos e EPSG:32632 após confirmação.
- Na sessão de teste do navegador, o fornecedor XYZ entregou parte das imagens e parte falhou; edição e alinhamento de coordenadas funcionaram com o satélite ativo. O aviso de imagens parciais foi ajustado para não desaparecer quando outro tile carrega. Disponibilidade de imagens depende da conexão e do fornecedor.
- Build, Clippy, instalação e atualização interna são conferidos antes da entrega; relatórios e hashes da distribuição ficam na pasta de entrega.

A edição do trajeto e os testes de software não substituem revisão operacional no navegador de bordo.
