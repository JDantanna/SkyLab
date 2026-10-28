# Consulta de atualização

O botão de atualização faz uma consulta manual às Releases públicas de [JDantanna/SkyLab](https://github.com/JDantanna/SkyLab/releases), sem configuração adicional. Ausência de Release e ausência de instalador são informadas explicitamente. Não há consulta periódica nem instalação automática.

Somente Releases estáveis, publicadas, com versão ou revisão interna posterior à instalada e um arquivo `SkyTEM-SkyLab-Setup-<versão>-r<revisão>.exe` hospedado neste repositório oferecem download. Tags sem Release não são atualizações. A versão aprovada permanece **0.1**. `BUILD_REVISION` identifica compilações publicadas da mesma versão; não altera o número 0.1 exibido no programa. A primeira compilação com esse controle usa revisão interna 1.

Instalações anteriores a esta correção não reconhecem revisões internas. É necessário instalar este pacote manualmente uma vez; a partir dele, correções futuras da 0.1 são reconhecidas pelo botão de atualização.

Para atualizar, clique no símbolo de atualização, obtenha o instalador, feche SkyLab e execute o arquivo baixado. O instalador preserva projetos pessoais. O botão não executa arquivos baixados automaticamente.

O fluxo `.github/workflows/release.yml` compila e testa o programa no Windows e publica o instalador com seu SHA256. É iniciado ao atualizar `BUILD_REVISION` em main, ao modificar o próprio fluxo, manualmente em **Actions → Windows installer → Run workflow**, ou por uma tag como `v0.1-r1`. A tag precisa corresponder a Cargo.toml e BUILD_REVISION. Uma Release existente nunca é substituída. Publicar uma revisão não autoriza avançar a versão 0.1. As próximas correções publicadas aumentam apenas BUILD_REVISION; uma mudança para 0.2 exige aprovação explícita.

Documentação da API utilizada: [GitHub Releases](https://docs.github.com/en/rest/releases/releases#get-the-latest-release).

Para mudar futuramente para um servidor da empresa, um administrador pode criar **updates.json** ao lado do executável instalado. Essa configuração tem prioridade sobre o GitHub:

```json
{"manifest":"https://servidor-da-empresa.example/SkyLab/manifest.json"}
```

O endereço acima é apenas ilustrativo e precisa ser substituído pelo endereço real. Também é possível indicar um arquivo, relativo à pasta da configuração:

```json
{"manifest":"versoes/manifest.json"}
```

O manifesto tem o formato:

```json
{
  "version":"0.1",
  "download_url":"https://servidor-da-empresa.example/SkyLab/instalador.exe",
  "notes":"Notas da versão aprovada."
}
```

`download_url` e `notes` são opcionais. A comparação é numérica e aceita versões com dois ou três componentes. Somente uma versão posterior exibe a opção de obter atualização. Essa opção abre o endereço HTTPS no navegador; não executa instaladores automaticamente.

A variável opcional SKYLAB_UPDATES_CONFIG permite indicar outra configuração, útil para testes ou implantação. A consulta HTTPS valida certificados, não envia levantamentos e tem limite de tempo e tamanho. Endereços HTTP, credenciais embutidas no endereço e destinos de download sem HTTPS são recusados. Autenticação corporativa específica ainda não foi implementada.

O manifesto corporativo deve listar somente versões aprovadas para distribuição.
