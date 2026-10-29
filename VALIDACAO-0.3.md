# Validação SkyLab 0.3 — revisão interna 1

## Verificações automatizadas

- Seleção Ctrl adiciona e retira uma linha; Shift mantém intervalo; Ctrl+Z restaura geometria e nomes.
- Sequência de nomes ordena índices selecionados, mantém os demais e recusa overflow, incremento inválido e nomes incompatíveis com MIS.
- Blocos: cálculo em Rust compara comprimento recortado, respeita furos, linhas externas e cruzamentos, sem modificar vértices. Cache por projeto imutável e respostas antigas não alteram outro documento.
- Fechamento de abas: operação ocupada informa motivo; confirmação não abre duas vezes; salvar cancelado mantém projeto com mensagem; salvar ou descartar fecha somente a aba escolhida.
- Atalho P aparece na configuração e preserva configurações antigas válidas.
- Compilação offline, análise Clippy sem avisos e testes nativos de janela e instalador.

## Conferência da interface

Projeto inteiramente fictício com dois blocos, seis linhas em cada um: painel muda de seis linhas L100xxx para seis L200xxx. Ctrl seleciona três linhas não consecutivas; prévia T00500/T00520/T00540 é aplicada somente nelas e Ctrl+Z restaura os nomes anteriores. P ativa caneta e F1 mostra sua configuração. Ajuda e Sobre aparecem após Painéis; Sobre mostra versão 0.3 e Jørgen Nissen. Fechar aba modificada apresenta confirmação.

## Critério de associação

Arquivos antigos não armazenam associação explícita linha/polígono. Usa-se o maior comprimento contido, com desempate pela ordem dos polígonos. Linhas externas continuam disponíveis pela camada. Esta associação filtra a lista e não recorta nem modifica os dados. Projetos JSON continuam compatíveis com os formatos existentes (1 a 4).

Resultados de software não constituem homologação operacional no navegador de bordo.
