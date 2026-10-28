# Validação — SkyLab 0.2, revisão interna 2

- 63 testes Rust: geometria, projeções, corte, camadas, arquivos recentes, seleção exportada, encaminhamento de segunda instância, recuperação, atualização e instalação.
- 25 testes de interação: inclui regressão de Ctrl+Z e Espaço temporário, devolução da ferramenta, seleção preservada e respeito à digitação.
- Dois cenários nativos de construção: aplicativo e instalador abrem a janela e carregam a logo.
- Compilação completa de aplicativo/instalador offline e clippy com warnings recusados.
- Conferência visual em 1280×720: logo transparente, recentes, confirmação de CRS, painel flutuante arrastável e redimensionável, ferramenta mão movendo o mapa e seleção preservada. Exportação pela interface conferida: três linhas selecionadas, quarta ausente no MIS.
- Instalação, reparo e desinstalação real de executáveis em pasta isolada: progresso, recusa de Origin externo e arquivos pessoais/configuração preservados.
- Atualização interna 0.1-r6 → 0.2-r2: pacote SHA256 e identidade compilada conferidos; reinício sem setup; três abas, aba ativa, todos os vértices, seleção, parâmetros, estado modificado e enquadramento preservados. Desinstalador migrado e executado; arquivo pessoal preservado.
- Registro MIS verificado no modo de instalação isolada, incluindo comando e remoção do registro simulado. Escrita no registro real foi recusada pelo ambiente de execução, portanto o teste direto da associação no Explorer deve ser feito sob o usuário Windows que instalará o programa. O código não altera UserChoice nem a escolha explícita de outro aplicativo.
- Painéis flutuam dentro da janela SkyLab, sem criar janelas nativas em outros monitores.
- Isto verifica software com dados sintéticos; não constitui homologação no navegador de bordo.
