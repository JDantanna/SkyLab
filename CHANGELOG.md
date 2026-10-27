# Histórico de versões

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
