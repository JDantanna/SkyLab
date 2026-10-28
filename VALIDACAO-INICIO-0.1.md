# Validação — revisão interna 4 da versão 0.1 — 28/10/2026

- Tela inicial verificada sem abas/projeto. Ctrl+N cria a primeira aba. Abrir e arrastar utilizam a confirmação de CRS antes de alterar o projeto. A última aba retorna à tela inicial.
- F1 abre a configuração dos seis atalhos na ordem solicitada. Alterar Q para X, rejeitar conflito com V e restaurar padrões foram conferidos na interface. Preferências persistem localmente.
- Créditos e e-mail conferidos no About. Adrian Mørch aparece duas vezes conforme a lista enviada pelo usuário.
- Corte vetorial por Q: 23 linhas / 41,28 km tornaram-se 33 linhas / 41,28 km; trechos cortados permaneceram selecionados. Ctrl+Z recuperou a geometria original.
- Google Satellite carregado no SkyLab sem chave pelo mesmo endpoint encontrado na instalação local do QuickMapServices/QGIS. Corrigida a ocultação do SVG vetorial ao ativar imagens. Corte sobre o satélite também preservou 41,28 km e selecionou os trechos resultantes. Sem validação aeronáutica ou de acurácia da imagem.
- O indicador confirmou salvamento automático no intervalo de 90 segundos. A cópia gerada pela própria interface recuperou o projeto sintético com 23 linhas, seleção e alterações não salvas.
- Teste com processos reais: encerramento forçado conserva o snapshot; reiniciar disponibiliza o estado; encerramento normal remove os arquivos próprios da recuperação. Gravação substitui a cópia anterior após sincronização e preserva arquivos desconhecidos.
- A transição tem handshake entre página e janela nativa: aguarda carregamento, fontes, decodificação da logo e dois quadros antes de revelar a página. Testes cobrem a ordem dos eventos; a aparência da animação na janela Windows deve ser conferida no computador do usuário.

Todos os levantamentos usados nos testes são sintéticos. Testes de software não substituem a validação operacional dos resultados no navegador de bordo.
