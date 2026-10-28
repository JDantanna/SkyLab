# Validação — edição e atualização interna — 28/10/2026

Versão exibida 0.1; revisão interna 3.

- Compilação release offline, testes da construção da janela nativa e Clippy com avisos tratados como erros aprovados.
- Geometria: curvas, múltiplas interseções, vértices compartilhados, traços repetidos, extremidades, sobreposições colineares, preservação da extensão e nomes únicos.
- Interface em execução: exemplo sintético de 23 linhas / 41,28 km dividido em 33 linhas / 41,28 km por arrasto; seleção retangular de nove trechos refletida na lista; exclusão resultou em 24 linhas; Ctrl+Z restaurou 33 e depois 23 linhas, com a extensão original.
- Atualização completa em instalação isolada: revisão 2 com novo protocolo para revisão 3, pacote local pelo mesmo verificador nativo. SHA256, identidade, aplicação por processo separado, encerramento do processo original, reinício e recuperação de três abas confirmados. Aba ativa, geometria não salva, seleção, enquadramento e formulário iguais aos originais.
- Preservação de arquivos pessoais, atalhos e desinstalador; retenção do executável anterior; limpeza temporária após restauração; estado sem atualização pendente na revisão 3.
- Recusa de pacote com hash incorreto e entrada ZIP de travessia aprovada. Falha simulada de registro recupera aplicativo e manifesto anteriores.

Testes usam projetos sintéticos em instalação isolada, sem substituir a instalação pessoal. A publicação e o transporte HTTPS são verificados separadamente. Isso não substitui conferência operacional no navegador de bordo. A extensão .mission é aceita como alternativa de nome para o mesmo formato textual MIS; outros formatos internos de equipamentos precisam de amostra e validação própria.
