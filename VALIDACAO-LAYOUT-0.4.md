# SkyLab 0.4 — revisão interna 3

Versão mantida em 0.4 conforme solicitado.

- Botões do fechamento de projeto na ordem Salvar e fechar, Fechar sem salvar e Voltar. Primeiro botão azul; três colunas iguais, ocupando toda a largura útil. Foco inicial em Salvar e fechar.
- Workspace identificado como Projeto; título acompanha o projeto ativo, mostra reticências quando necessário e o nome completo no tooltip. Nomes de projeto são preservados na troca de idioma.
- Abas sem margem superior, sem padding horizontal externo e sem folga inferior. Geometria DOM verificada: primeira aba x=0, topo=50 coincidindo com o fim do header, altura=36 coincidindo com a barra de abas.
- Verificação visual em português, inglês e dinamarquês; retorno do diálogo mantém o projeto aberto. Título longo de teste permaneceu limitado à largura disponível. Nova aba atualizou o título para Novo projeto 2.
- scripts/build.ps1: inicialização nativa do programa/instalador, testes JavaScript e Rust, compilação release.
- cargo clippy --all-targets --features installer --offline -- -D warnings.
- Instalação e atualização interna são verificadas separadamente na entrega, utilizando dados sintéticos e --test-root; preservação de arquivos pessoais, seleção e abas.

A publicação e hashes do instalador constam na entrega local.
