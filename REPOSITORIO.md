# Repositório e continuidade

O código está em um **repositório Git local**, na mesma pasta deste documento. A branch principal é `main`. A versão aprovada é **0.1**. A base inicial permanece em `v0.1.0`; a revisão atual é identificada como `revisao-otimizacao-0.1`. A antiga marcação 0.2 foi retirada como versão corrente, conforme orientação do usuário, e preservada apenas no histórico preliminar.

O destino remoto `origin` está configurado para **https://github.com/JDantanna/SkyLab.git**, por solicitação do usuário. O repositório de destino está público. Na preparação desta migração, o destino estava vazio e o conector confirmou acesso de escrita; o primeiro envio ainda aguarda autenticação do Git na sessão do Windows. O acesso do conector GitHub não configura automaticamente credenciais do Git local. O arquivo Publicar-SkyLab-no-GitHub.cmd, entregue na pasta outputs, executa o envio de main e tags sem force push, depois do login normal do usuário. Não contém credenciais.

Para continuar a manutenção, abra esta pasta como projeto local do aplicativo e peça as alterações neste chat ou em outro chat desse projeto. É possível revisar diferenças e histórico no Git. A pasta contém o código, testes, scripts, marca e dependências offline; os dados reais de levantamento não fazem parte do repositório.

## Cópia migrável

A entrega inclui um arquivo `.bundle` com todo o histórico e as tags. Para restaurar em outra máquina com Git:

```text
git clone SkyLab-repositorio-0.1-otimizado.bundle SkyLab
```

O bundle é uma fotografia do histórico nesta entrega; as próximas alterações ficam no repositório local. Gere um novo bundle quando precisar transportar um histórico atualizado:

```text
git bundle create SkyLab-atualizado.bundle --all
```

O destino atual é JDantanna/SkyLab no GitHub. A futura migração para um servidor da empresa pode manter o histórico existente; não é necessário recriar o projeto. Projetos e levantamentos confidenciais, instaladores locais e pastas de trabalho não integram os arquivos versionados para envio. Os arquivos de certificado/chave em vendor/tiny_http/examples pertencem aos exemplos públicos da dependência e conferem com o checksum do pacote original; não são credenciais da empresa.

## Versões do programa

Alterar o código neste repositório não atualiza os computadores dos operadores automaticamente. Cada versão deve ser compilada e testada; depois, distribui-se o novo instalador. Ao executá-lo sobre uma instalação reconhecida, ele oferece atualização/reparo e preserva os arquivos pessoais.

O botão Checar atualização faz uma consulta manual à fonte configurada pela empresa; consulte ATUALIZACOES.md. Sem essa configuração, a interface informa que a fonte ainda não foi definida. O programa não instala atualizações automaticamente.

A cada pedido de modificação, deve-se perguntar se há autorização para avançar a versão. Sem autorização explícita, os ajustes permanecem em 0.1. Histórico de commits não implica aumento de versão do produto.
