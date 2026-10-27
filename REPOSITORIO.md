# Repositório e continuidade

O código está em um **repositório Git local**, na mesma pasta deste documento. A branch principal é `main`. Há uma base marcada como `v0.1.0` e uma versão com identidade SkyTEM e instalador marcada como `v0.2.0`.

Este repositório fica no espaço de trabalho local usado por este chat. Não foi publicado em um servidor remoto. Projetos e pastas locais são suportados pelo ambiente GPT/Codex; isso não constitui, por si só, hospedagem Git remota. Consulte a [documentação oficial de projetos](https://learn.chatgpt.com/docs/projects).

Para continuar a manutenção, abra esta pasta como projeto local do aplicativo e peça as alterações neste chat ou em outro chat desse projeto. É possível revisar diferenças e histórico no Git. A pasta contém o código, testes, scripts, marca e dependências offline; os dados reais de levantamento não fazem parte do repositório.

## Cópia migrável

A entrega inclui um arquivo `.bundle` com todo o histórico e as tags. Para restaurar em outra máquina com Git:

```text
git clone SkyLab-Rust-repositorio-v0.2.0.bundle SkyLab
```

O bundle é uma fotografia do histórico nesta entrega; as próximas alterações ficam no repositório local. Gere um novo bundle quando precisar transportar um histórico atualizado:

```text
git bundle create SkyLab-atualizado.bundle --all
```

Quando a empresa definir o servidor e as permissões, o mesmo repositório pode receber um remoto corporativo e enviar a branch e as tags. Não é necessário recriar o projeto ou perder o histórico. O destino remoto deve ser informado antes dessa publicação.

## Versões do programa

Alterar o código neste repositório não atualiza os computadores dos operadores automaticamente. Cada versão deve ser compilada e testada; depois, distribui-se o novo instalador. Ao executá-lo sobre uma instalação reconhecida, ele oferece atualização/reparo e preserva os arquivos pessoais.

Um atualizador automático poderá ser implementado quando houver um servidor de distribuição, política de versões e assinatura de código da empresa. Esta entrega não faz consultas de atualização na rede.
