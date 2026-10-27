# Repositório e continuidade

O código está em um **repositório Git local**, na mesma pasta deste documento. A branch principal é `main`. A versão aprovada é **0.1**. A base inicial permanece em `v0.1.0`; a revisão atual é identificada como `revisao-layout-0.1`. A antiga marcação 0.2 foi retirada como versão corrente, conforme orientação do usuário, e preservada apenas no histórico preliminar.

Este repositório fica no espaço de trabalho local usado por este chat. Não foi publicado em um servidor remoto. Projetos e pastas locais são suportados pelo ambiente GPT/Codex; isso não constitui, por si só, hospedagem Git remota. Consulte a [documentação oficial de projetos](https://learn.chatgpt.com/docs/projects).

Para continuar a manutenção, abra esta pasta como projeto local do aplicativo e peça as alterações neste chat ou em outro chat desse projeto. É possível revisar diferenças e histórico no Git. A pasta contém o código, testes, scripts, marca e dependências offline; os dados reais de levantamento não fazem parte do repositório.

## Cópia migrável

A entrega inclui um arquivo `.bundle` com todo o histórico e as tags. Para restaurar em outra máquina com Git:

```text
git clone SkyLab-repositorio-0.1-layout.bundle SkyLab
```

O bundle é uma fotografia do histórico nesta entrega; as próximas alterações ficam no repositório local. Gere um novo bundle quando precisar transportar um histórico atualizado:

```text
git bundle create SkyLab-atualizado.bundle --all
```

Quando a empresa definir o servidor e as permissões, o mesmo repositório pode receber um remoto corporativo e enviar a branch e as tags. Não é necessário recriar o projeto ou perder o histórico. O destino remoto deve ser informado antes dessa publicação.

## Versões do programa

Alterar o código neste repositório não atualiza os computadores dos operadores automaticamente. Cada versão deve ser compilada e testada; depois, distribui-se o novo instalador. Ao executá-lo sobre uma instalação reconhecida, ele oferece atualização/reparo e preserva os arquivos pessoais.

O botão Checar atualização faz uma consulta manual à fonte configurada pela empresa; consulte ATUALIZACOES.md. Sem essa configuração, a interface informa que a fonte ainda não foi definida. O programa não instala atualizações automaticamente.

A cada pedido de modificação, deve-se perguntar se há autorização para avançar a versão. Sem autorização explícita, os ajustes permanecem em 0.1. Histórico de commits não implica aumento de versão do produto.
