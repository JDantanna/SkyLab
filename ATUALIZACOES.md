# Atualização interna

O símbolo de atualização consulta as Releases estáveis de [JDantanna/SkyLab](https://github.com/JDantanna/SkyLab/releases). Quando houver uma compilação posterior, **Update and restart** baixa o pacote do aplicativo, verifica SHA256 e identidade, salva a sessão, encerra, substitui o executável e reinicia. Não abre navegador nem outro instalador. Não há consulta periódica nem telemetria.

A versão aprovada é **0.3**. BUILD_REVISION identifica correções da mesma versão; avançar para 0.4 requer aprovação explícita. Publicações são imutáveis, identificadas por v0.3-r2, por exemplo.

**Instalações 0.1, revisão interna 2 ou anteriores, precisam executar o instalador uma vez.** Elas ainda possuem o mecanismo antigo; o novo comportamento passa a valer após essa instalação.

## Preservação da sessão

Até três abas, projetos não salvos, seleção, enquadramento e valores dos campos são recuperados. O histórico de desfazer começa uma nova sessão após reiniciar. Pacote e sessão ficam temporariamente no computador e são removidos após a confirmação da restauração. Nenhum levantamento é enviado ao GitHub.

SkyLab-Rust.previous.exe mantém o executável anterior. Falha na substituição ou no registro restaura executável e manifesto anteriores. Atalhos e arquivos pessoais permanecem. Ao migrar da 0.1, o desinstalador também é atualizado para remover os registros MIS próprios. O aplicativo precisa estar instalado; cópias portáteis não são atualizadas por esse fluxo.

## Publicação

.github/workflows/release.yml testa e compila no Windows. Publica instalador, SkyLab-Update-<versão>-r<revisão>.zip e SHA256SUMS. O ZIP contém apenas SkyLab-Rust.exe. O cliente confere o digest SHA256 da API do GitHub, recusa entradas extras/caminhos de travessia e consulta a identidade do binário antes de encerrar. HTTPS valida certificados; origem inicial restrita às Releases deste repositório e redirecionamentos restritos aos hosts de distribuição do GitHub.

O fluxo inicia ao modificar BUILD_REVISION, testes ou o fluxo em main; também pode ser iniciado manualmente ou por tag correspondente à versão/revisão. Releases existentes não são substituídas.

## Configuração opcional

updates.json ao lado do executável, ou SKYLAB_UPDATES_CONFIG, pode indicar um manifesto local ou HTTPS:

```json
{"manifest":"versoes/manifest.json"}
```

```json
{
  "version":"0.1",
  "revision":4,
  "package_url":"https://github.com/JDantanna/SkyLab/releases/download/v0.1-r4/SkyLab-Update-0.1-r4.zip",
  "package_sha256":"<SHA256 de 64 caracteres hexadecimais>",
  "notes":"Correções da versão aprovada."
}
```

Exemplo ilustrativo. A consulta aceita uma fonte corporativa; a instalação interna continua restrita aos pacotes do repositório autorizado. Migrar para servidor da empresa exigirá configurar e validar a nova origem. Autenticação corporativa não está implementada.
