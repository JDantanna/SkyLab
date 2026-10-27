# Validação da revisão 0.1 — abas e prévias

Versão mantida em 0.1 (Cargo 0.1.0), conforme orientação expressa. Apenas dados sintéticos usados.

- `scripts/build.ps1`: 39 testes Rust e 8 testes de interação aprovados; aplicativo e instalador release compilados.
- Clippy com todos os alvos e recurso installer: aprovado, sem avisos.
- Interface verificada no navegador local: menu na logo, cabeçalho de 50 px, limite de três abas, retorno ao projeto preservando espaçamento, seleção e desfazer/refazer. Campos de numeração alinhados em português.
- Espaçamento de 200 para 300 e 400 m mostrou prévias de 15 e 11 linhas sem modificar as 23 linhas originais. Slider atualiza o campo e o desenho. Prévia cancelada ao mudar de aba.
- Corte inicial de 200 m mostrou prévia de 1.630,6 m; aplicar alterou o projeto; Ctrl+Z restaurou 1.830,6 m.
- Testes de concorrência confirmam descarte de respostas antigas e cálculo único de prévia por vez. A prévia nunca entra no histórico do projeto.
- Instalador em `--test-root`: instalação, reparo, versão 0.1, hash do aplicativo instalado, atalhos, desinstalação própria, preservação de projeto/configuração e recusa de origem externa aprovados.
- Parser de Releases: versão numérica, drafts/prereleases ignorados, instalador obrigatório e restrito ao repositório autorizado, ausência de Release diferenciada de aplicativo atualizado.

A API pública do GitHub retornou 404 para a última Release: ainda não existe Release publicada. O fluxo de publicação foi preparado, mas não executado no GitHub durante esta validação. A consulta HTTPS pelo adaptador nativo encontrou `Credenciais não disponíveis no pacote de segurança` no contexto gerenciado de execução; a consulta pública pelo outro cliente funcionou. A validação TLS foi preservada. A consulta nativa precisa ser conferida na sessão Windows do usuário.

O splash com logo e a reserva dos controles nativos foram implementados no adaptador Windows; a inspeção visual desta revisão foi feita no conteúdo local da interface. O diálogo nativo de salvar e o splash não receberam nova verificação visual nesta revisão.

As abas são da sessão atual. Projetos precisam ser salvos individualmente para reabertura; não há recuperação automática das abas após encerrar o aplicativo. Revisões que continuam em 0.1 não são oferecidas como versões posteriores pelo atualizador.

Não constitui homologação de uso em voo.
