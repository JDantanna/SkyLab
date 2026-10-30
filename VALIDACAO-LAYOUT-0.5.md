# SkyLab 0.5 — painéis e cartografia, revisão interna 3

Versão visível mantida em 0.5 por solicitação expressa.

## Alterações

- Removidos integralmente os painéis Selected line e Actions for all lines: registro dos painéis, opções do seletor, controles HTML, ações exclusivas, estilos e referências. Layouts salvos com os IDs antigos são ignorados pelo restaurador existente.
- Cortar por distância continua independente. Renomear, inverter, excluir, recortar e editar geometria continuam disponíveis nas ferramentas e menus de contexto existentes.
- Abrir/criar projeto pede maximização à janela WebView2. O comando é idempotente: duas solicitações não restauram a janela. Alternar abas existentes ou adicionar uma camada não força nova maximização.
- Cartografia passa a ter um botão com ícone de mapa imediatamente após Editar. O menu principal deixa de incluir Cartografia. Atalho Shift+C e personalização do atalho permanecem; a dica do botão acompanha a configuração.
- Restauração de sessões antigas ignora campos dos controles removidos.

## Verificações locais

- scripts/build.ps1 aprovado: 56 testes de interface e 81 testes Rust, além da inicialização nativa do aplicativo e instalador.
- cargo clippy --all-targets --features installer --offline -- -D warnings aprovado.
- Teste nativo verifica duas solicitações de maximização consecutivas sem voltar ao modo janela.
- Teste de documentos verifica maximização ao criar/abrir, sem repetir ao alternar abas existentes. Teste de importação confirma a solicitação após abrir arquivo em outra zona UTM.
- Conferência da interface com sessão sintética anterior contendo campos line-name e points: restauração concluída, painéis retirados ausentes também do seletor Painéis e cartografia ausente do menu principal.
- Clique no novo ícone abre cartografia; Shift+C retorna ao layout com painéis. Cortar por distância divide uma linha em trechos de 500,0 m e 1.014,3 m, mantendo 16,91 km de total; desfazer restaura as 12 linhas anteriores.
- Busca por referências aos IDs dos controles removidos não encontrou ligações JavaScript restantes.

Nenhum levantamento real foi utilizado ou incluído na distribuição. Os resultados de instalação e atualização do pacote público acompanham a entrega local em arquivos JSON.
