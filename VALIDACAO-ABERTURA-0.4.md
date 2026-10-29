# SkyLab 0.4 — revisão interna 2

Versão mantida em 0.4 por aprovação explícita. Revisão de distribuição: 2.

## Correções verificadas

- Importação de outra zona/hemisfério usa o sistema confirmado. Se a geometria anterior não puder ser reprojetada, o arquivo abre em outra aba e a interface informa o motivo; três abas ocupadas recebem instrução para fechar uma. Nenhuma geometria anterior é descartada.
- Erro de abertura aparece em alerta na tela inicial e no status do workspace.
- Arrastar arquivos no centro aceita múltiplos arquivos, impede importação duplicada e conserva a ponte de arquivos nativa.
- Menus de linha do mapa, Mission lines e Project layers oferecem Cortar por distância; o painel é reaberto e expandido.
- Distância editada em De/Até sincroniza o ponto de divisão; redesenho não restaura valores padrão. O campo Dividir na distância também pode ser editado diretamente.
- Divisão transacional Rust preserva vértices intermediários, ponto comum, comprimento total, camada e visibilidade; nomes não colidem. As duas partes ficam selecionadas e Ctrl+Z restaura a linha.
- Cabeçalhos usam uma seta exclusiva para recolher/expandir, sem seletor de posição. Reposicionamento continua pelo arrasto. Seis símbolos SVG locais identificam os painéis.

## Verificação

- scripts/build.ps1: inicialização nativa do programa e instalador; testes JavaScript e Rust; compilação release de ambos.
- cargo clippy --all-targets --features installer --offline -- -D warnings.
- Regressões novas: mudança UTM, limite de abas sem perda, erro visível, arrasto múltiplo, conservação dos campos e sincronização do limite.
- HTTP Rust: divisão no vértice e dentro de segmento, limites inválidos, preservação de comprimento e colisão de nomes.
- Interface com dados sintéticos: 1.400 m dividido em 375 m + 1.025 m, duas seleções, total 16,80 km preservado; Desfazer retornou de 13 para 12 linhas. Ícones presentes e seletor de posição ausente.
- Instalação, reparo e desinstalação em --test-root preservaram arquivos pessoais.
- Atualização interna de 0.4-r1 para 0.4-r2 verificou hash e identidade do pacote, reiniciou sem setup e restaurou três abas, seleção, formulário, estado não salvo e enquadramento; desinstalador e atalhos preservados.

Os dados usados nos testes são sintéticos. A publicação e seus hashes são registrados separadamente na entrega local.
