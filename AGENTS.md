# SkyLab — instruções de manutenção

- Este repositório contém o código e a marca autorizada para esta versão interna. Os levantamentos reais são confidenciais: não copiar para o repositório, não incluir em pacotes e não enviar a serviços externos.
- Manter cálculos e validações geométricas em Rust. A interface e o instalador usam recursos locais; não adicionar telemetria. Por solicitação expressa do usuário, a camada de satélite Google é permitida somente após ativação/configuração pelo operador; o funcionamento vetorial deve permanecer offline.
- A zona/hemisfério são explícitos. Não inferir datum por nome de arquivo nem transformar LineString em bloco automaticamente. Preservar todos os vértices MIS.
- Usar assets/skylab-icon.svg e assets/app.ico: marca SkyLab com o S original, frasco e linha, aprovada para aplicação pelo usuário. Preservar assets/skytem-white.svg como fonte do desenho do S. Consultar IDENTIDADE.md.
- Para distribuir, executar scripts/build.ps1, testar o instalador em modo --test-root e verificar instalação/atualização/desinstalação preservando arquivos pessoais. Compilação é offline; Windows SDK fornece rc.exe para ícone/metadados.
- Executar cargo clippy --all-targets --features installer --offline -- -D warnings depois que payload/SkyLab-Rust.exe tiver sido gerado.
- Atualizar Cargo.toml, CHANGELOG.md e os relatórios de validação ao criar uma versão. A versão de registro do instalador vem de Cargo.toml.
- A versão aprovada explicitamente para esta build é 0.2 (internamente 0.2.0). Em cada novo pedido de modificações, perguntar se deseja avançar a versão e só incrementar após resposta explícita.
- Os avanços seguem 0.1, 0.2, 0.3; não saltar para 1.0 por iniciativa própria.
- BUILD_REVISION é somente o identificador interno das correções publicadas; aumentar ao publicar uma nova compilação da mesma versão. O número exibido permanece 0.2 até nova aprovação expressa. O fluxo de Releases publica a combinação imutável versão/revisão.
- Interface institucional: nome, marca e comandos diretos. Não incluir slogans, chamadas de venda ou mensagens promocionais. Usar janela WebView2 própria com controles integrados; não voltar ao Edge em modo aplicativo.
- Guardar artefatos em dist/ ou como entregas locais; não versionar executáveis, pastas temporárias, payload ou levantamentos.
- Histórico em main, com remoto autorizado https://github.com/JDantanna/SkyLab. Atualizações consultam Releases desse repositório; tags sozinhas não distribuem instaladores. Não afirmar que existe atualização automática no GPT.
- Resultados precisam de validação operacional no navegador de bordo antes de uso em voo. Não apresentar testes de software como homologação aeronáutica.
