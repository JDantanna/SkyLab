# Validação — camadas e controles — 28/10/2026

Versão exibida 0.1; revisão interna 5.

- Interface: criou New project 1, fechou a última aba, retornou à tela inicial e criou New project 1 novamente. Uma aba adicional recebeu New project 2. Identificadores internos continuam únicos; rótulos independem deles.
- Importação real pela interface de três arquivos sintéticos escolhidos juntos: Area.kmz, Reference.mis e Flown.mis. CRS confirmado em cada importação. As três camadas apareceram separadas, com um polígono e quatro linhas em cada MIS. Nomes iguais entre os dois MIS foram mantidos.
- Expansão da camada revelou suas linhas. Botão direito → Export layer MIS gerou um arquivo idêntico, byte a byte, ao Reference.mis sintético: quatro linhas, sem as quatro linhas de Flown.mis.
- Arrasto do rótulo Spacing: valor 100 para 170 ao mover 70 pixels à direita; depois 130 ao mover 40 pixels à esquerda. A prévia mostrou 27 e depois 35 linhas. Nenhum painel flutuante foi aberto.
- Troca de Google Satellite para Hybrid com imagem ativa: todos os blocos visíveis mudaram de lyrs=s para lyrs=y, conservando exatamente índices X/Y, zoom e posições na tela. Desligar é separado.
- Hover no menu: botão com transformação none, borda zero, fundo transparente e sombra none; somente a imagem teve escala 1,08. Espaçamento visual entre título, ferramentas e indicadores conferido.
- Testes de geometria: exportação por camada, isolamento de IDs, conservação da camada e vértices em corte/trim, JSON de ida e volta, validação de referências e leitura de projetos antigos.
- KMZ: leitura de doc.kml sem extração; recusa de arquivos ambíguos, base64 inválido e conteúdo sem ZIP. Limites de tamanho e quantidade de entradas aplicados.

Os dados usados são sintéticos. A validação de software não homologa arquivos para o navegador de bordo; a revisão operacional continua necessária.
