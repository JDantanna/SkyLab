# Correção de abertura — SkyLab 0.1

O erro informado foi reproduzido com o mesmo ícone e o .NET Framework utilizado pelo programa. `Icon.ToBitmap()` lançou `ArgumentOutOfRangeException: Intervalo solicitado ultrapassa o fim da matriz`, em `System.Drawing.Icon.BmpFrame()`.

A substituição por `Bitmap.FromHicon()` preserva a logo e utiliza a renderização nativa do Windows. O teste reproduzindo o método antigo falhou; o método corrigido produziu a imagem de 128 × 128 px.

Validação do novo pacote:

- Criação real dos dois formulários nativos, aplicativo e instalador, com o mesmo código de produção: aprovada. A imagem do splash foi verificada quanto a tamanho e conteúdo não transparente. Não depende de abrir a interface em um navegador externo.
- 39 testes Rust e 8 testes de interação: aprovados.
- Compilação release do aplicativo e do instalador: aprovada.
- Clippy com todos os alvos e recurso installer, sem avisos: aprovado.
- Instalação em pasta de teste, hash do aplicativo, reparo, registro da versão 0.1, atalhos, desinstalação e preservação de projetos/configurações: aprovados.

O teste nativo constrói os formulários sem iniciar a navegação WebView2. A tentativa de localizar a janela completa do instalador pela automação do desktop não encontrou uma janela própria do instalador; portanto, não é registrada como verificação visual completa da abertura. A causa específica do erro relatado foi reproduzida e coberta pelo teste de regressão.

Versão mantida em 0.1 (Cargo 0.1.0), conforme confirmação do usuário. Não há necessidade de reinstalar WebView2 para corrigir esta exceção de conversão do ícone.
