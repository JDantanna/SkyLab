# Google Maps: opções de uso sem cobrança

Consulta às fontes oficiais em 27/10/2026. Preços e franquias podem mudar; a conta da empresa deve ser conferida antes da ativação.

## Editor atual: Maps JavaScript API

O SkyLab usa o mapa dinâmico oficial, com imagem de satélite e OverlayView para manter as linhas alinhadas. A tabela global atual oferece **10.000 eventos mensais gratuitos para Dynamic Maps**. A cobrança começa ao ultrapassar a franquia aplicável. É necessário habilitar faturamento e fornecer uma chave da API, mesmo quando o uso permanece gratuito.

Portanto, é possível operar sem cobrança dentro da franquia, mas não existe garantia de gratuidade ilimitada. O uso é agregado entre projetos da conta de faturamento segundo as regras do Google. A TI deve conferir todos os consumidores da conta, restringir a chave, desabilitar APIs desnecessárias e configurar cotas quando disponíveis. Alertas de orçamento apenas avisam; não são um bloqueio automático de cobrança. Não foi configurada nenhuma conta, chave, assinatura ou faturamento nesta entrega.

O SkyLab mantém a mesma instância do mapa ao desligar/religar a camada na mesma sessão, evitando recriar o mapa desnecessariamente. A camada permanece opcional e desligada ao abrir o programa. Os arquivos locais e a camada vetorial funcionam sem Google.

Fontes: [tabela de preços](https://developers.google.com/maps/billing-and-pricing/pricing), [uso e faturamento JavaScript](https://developers.google.com/maps/documentation/javascript/usage-and-billing), [controle de uso](https://developers.google.com/maps/billing-and-pricing/manage-costs).

## API gratuita distinta: Maps Embed

A Maps Embed API tem uso gratuito e ilimitado, exigindo chave. Ela oferece um mapa incorporado. Não substitui diretamente a integração atual de edição com OverlayView e controle de câmera; não foi implementada como fundo do editor. Uma referência separada por mapa incorporado pode ser avaliada se esse fluxo for desejado.

Fonte: [Maps Embed — uso e faturamento](https://developers.google.com/maps/documentation/embed/usage-and-billing).

## Acesso direto ao site do Google

A documentação oficial informa que imagens e tiles devem ser acessados por Google Maps Platform. Usar URLs internas/não documentadas do site para contornar a cobrança não é uma alternativa autorizada pelo Google. Não foram adicionadas conexões desse tipo ao programa.

Fonte: [FAQ oficial — acesso a tiles/imagens](https://developers.google.com/maps/faq).

## Situação desta entrega

A integração existente continua disponível para uma chave habilitada da empresa. Nenhuma imagem real foi consultada nem enviada missão ao Google durante os testes. Não há compromisso de cobrança nem chave embutida no instalador. Se o requisito for custo obrigatoriamente zero independentemente do volume, a integração atual depende de uma política de cotas da conta; alternativamente será necessário escolher uma fonte de imagens cuja licença e condições atendam a esse requisito.
