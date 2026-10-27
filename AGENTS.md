# SkyLab — instruções de manutenção

- Este repositório contém o código e a marca autorizada para esta versão interna. Os levantamentos reais são confidenciais: não copiar para o repositório, não incluir em pacotes e não enviar a serviços externos.
- Manter cálculos e validações geométricas em Rust. A interface e o instalador usam recursos locais; não adicionar mapas remotos, telemetria ou dependências de rede em execução.
- A zona/hemisfério são explícitos. Não inferir datum por nome de arquivo nem transformar LineString em bloco automaticamente. Preservar todos os vértices MIS.
- Manter a logo original em assets/skytem-logo.jpg. Não redesenhar ou trocar por uma marca gerada. Consultar IDENTIDADE.md.
- Para distribuir, executar scripts/build.ps1, testar o instalador em modo --test-root e verificar instalação/atualização/desinstalação preservando arquivos pessoais. Compilação é offline; Windows SDK fornece rc.exe para ícone/metadados.
- Executar cargo clippy --all-targets --features installer --offline -- -D warnings depois que payload/SkyLab-Rust.exe tiver sido gerado.
- Atualizar Cargo.toml, CHANGELOG.md e os relatórios de validação ao criar uma versão. A versão de registro do instalador vem de Cargo.toml.
- Guardar artefatos em dist/ ou como entregas locais; não versionar executáveis, pastas temporárias, payload ou levantamentos.
- Histórico local em main, com tags de versões testadas. O destino remoto de hospedagem ainda não foi definido. Não afirmar que existe hospedagem ou atualização automática no GPT.
- Resultados precisam de validação operacional no navegador de bordo antes de uso em voo. Não apresentar testes de software como homologação aeronáutica.
