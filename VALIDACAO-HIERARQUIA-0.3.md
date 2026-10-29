# SkyLab 0.3 — revisão interna 2

## Cobertura

- Grupos aninhados, reordenação, retorno à raiz, renomeação e remapeamento de IDs ao importar um projeto; coordenadas e origem de arquivos permanecem intactas.
- Visibilidade herdada de grupos, controle individual de linhas e roundtrip do projeto JSON formato 5; metadados opcionais mantêm leitura de versões anteriores.
- Rust recusa ciclos, pais ausentes ou que não sejam grupos, geometria associada diretamente a uma pasta e hierarquia em formato antigo.
- Corte manual respeita geometria oculta, mantém vértices e não muda linhas escondidas. Polígonos editáveis mantêm formato 5 e novos desenhos usam uma camada visível.
- Alterações de organização e visibilidade mantêm seleção vazia; respostas antigas da classificação de blocos não mudam outra aba. O cache da associação geométrica é reutilizado em mudanças apenas de organização.
- Teste de interface com projeto fictício de dois blocos e 12 linhas: arrastar arquivo para grupo, retirar para a raiz, ocultar e reexibir; ocultação mantém 12 linhas no projeto e mostra somente as seis do outro bloco.
- Cliques repetidos para expandir/recolher camadas não perdem a expansão.
- Botões de exportação ficam dentro do painel; exportação geral mostra 12 linhas, enquanto a seleção usa o caminho MIS de índices selecionados.
- Painel flutuante fica ao lado do polígono. Campos e botões conferidos em português, sem exceder os limites da caixa; cursores distintos conferidos nas ferramentas Q, V e P. Prévia posterior usa violeta sobre as linhas azuis existentes.

## Persistência e limites

A organização é por arquivo/camada e pastas; os dados completos permanecem no mapa e no projeto. Linhas exibidas continuam filtradas pelo bloco ativo. Um grupo pai oculto mantém os filhos ocultos até ser reexibido. Ocultar afeta a visualização, não a exportação geral.

O formato 5 impede que leitores antigos ignorem silenciosamente a hierarquia e a visibilidade. Telas menores podem rolar o conteúdo dos painéis; campos mantêm seus limites ao redimensionar. Validação de software não equivale a homologação no navegador de bordo.
