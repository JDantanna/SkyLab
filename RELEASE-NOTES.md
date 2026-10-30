# SkyLab 0.5

- Modo cartografia: Shift+C, workspace ampliado e barra lateral flutuante.
- Edição de linhas por pontos e alças Bézier, corte, extensão e emenda com prévia; Ctrl+Z/Ctrl+Y.
- Satélite e enquadramento acessíveis na barra; Espaço mantém navegação durante edição.
- Importação de projetos QGIS .qgs/.qgz/.qlr com fontes vetoriais locais GeoPackage, Shapefile, GeoJSON e KML/KMZ, preservando grupos e coordenadas.
- Fontes referenciadas devem continuar disponíveis junto ao projeto. Fontes ausentes/incompatíveis geram aviso; não são executados plugins ou conexões remotas do QGIS.
- Salvar mantém o projeto em .skylab. MIS exporta a geometria validada e achatada, sem remover vértices.
- Atualização interna preserva os projetos abertos. Versão de testes institucional.
