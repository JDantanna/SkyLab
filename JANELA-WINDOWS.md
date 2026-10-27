# Janela Windows

O aplicativo e o instalador mantêm o cálculo, a validação e a instalação em Rust. A janela é hospedada pelo Microsoft WebView2 por meio de um adaptador .NET Framework pequeno, compilado de native/Host.cs. HTML/CSS/JavaScript continuam incorporados nos executáveis.

A janela usa FormBorderStyle.None: não há a barra de título cinza do navegador. Minimizar, maximizar/restaurar e fechar são botões nativos desenhados em azul, acima da superfície WebView2. Permanecem acessíveis quando há diálogos abertos. O cabeçalho permite mover a janela. O aplicativo encerra o processo local quando a janela principal termina. Uma segunda abertura ativa a instância existente.

Os componentes do SDK WebView2 estão fixados em native/vendor/webview2, junto da licença e dos avisos da Microsoft. O SDK foi obtido do pacote oficial Microsoft.Web.WebView2 1.0.4314.35. Foram incluídas somente as bibliotecas .NET Framework e o carregador x64 necessários.

A máquina de uso precisa de **Windows 64 bits, .NET Framework 4.6.2 ou posterior e Microsoft Edge WebView2 Runtime**. O runtime já estava instalado no computador desta sessão. O instalador não o baixa automaticamente; a TI deve disponibilizá-lo em máquinas que ainda não o tenham. Com esses pré-requisitos, a operação local não exige internet.

O adaptador e as bibliotecas de suporte são materializados num cache temporário por compilação. O perfil WebView2 permanece local em %LOCALAPPDATA%/SkyTEM/SkyLab/WebView2. O desinstalador preserva projetos e configurações pessoais; não faz limpeza recursiva de perfis. Nenhum arquivo de levantamento é incluído no pacote.

Para compilar são necessários Rust, Windows SDK e o compilador .NET Framework disponível no Windows. Não é necessário restaurar pacotes pela internet: dependências e licenças estão no repositório. As variáveis opcionais SKYLAB_NATIVE_CACHE, SKYLAB_WEBVIEW_PROFILE e SKYLAB_DOWNLOADS permitem isolar caches e downloads nos testes.

Referências: documentação oficial de WebView2/WinForms e distribuição do runtime, consultadas na implementação.

[WebView2 em WinForms](https://learn.microsoft.com/en-us/microsoft-edge/webview2/get-started/winforms) · [Distribuição do runtime](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution)
