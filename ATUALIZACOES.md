# Consulta de atualização

O botão **Checar atualização** faz uma consulta manual. Sem uma fonte configurada, informa isso explicitamente; não afirma que o aplicativo está atualizado. Não há consulta periódica nem instalação automática.

A fonte pode ser um manifesto JSON em servidor HTTPS ou um arquivo local/compartilhado. Um administrador deve criar **updates.json** ao lado do executável instalado, com:

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

A versão atualmente aprovada é **0.1**. O manifesto deve listar somente versões aprovadas para distribuição. O botão não consulta tags Git ou as antigas entregas preliminares. O servidor/arquivo da empresa ainda precisa ser definido.
