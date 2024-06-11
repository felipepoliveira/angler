# Como rodar o Angler

O Angler é um serviço escalável que possui recursos para distribuição de carga computacional em clusters. Pensando nisso e em facilitar a experiência dos desenvolvedores para publicar o Angler do modo mais fácil simplificamos os modos de execução do Angler para que ele possa executar ou em um único nó de processamento ou em vários.

## Executando o Angler no Modo Standalone

_Powershell_
```ps
angler.exe
```

ou

_Powershell_
```ps
angler.exe --controller --broker
```

Ao executar o aplicativo neste modo ele estará pronto para uso.

## Modos de Execução do Angler

### Controller

Neste modo o Angler fará o papel de nó principal que receberá as requisições de todos os clientes. Caso outros nós participarem do cluster desta instância do Angler, ele sera o coordenador de balancamento de carga das instâncias. Além disso, o _controller_ também é responsável por armazenar os registros das mensagens enviadas através do serviço, onde é possível, por exemplo, utilizar a API de clientes do Angler para buscar as mensagens enviadas, com problemas de envio ou as não enviadas.

_Powershell_
```ps
angler.exe --controller
```

### Broker

Neste modo a instância só servirá para participar de um _cluster_ de processamento Angler. Para isso será necessário definir no arquivo de configuração do Angler as informações do Cluster, tais como _host_ do _controller_ e a chave de acesso ao _cluster_.

_Powershell_
```ps
angler.exe --broker
```

### Argumentos da Aplicação
| Nome      | Tipo          |   Descrição   |
|-          |-              |-              |
| broker  | --flag    | Define se a instância do angler rodara em modo _broker_.
| controller  | --flag    | Define se a instância do angler rodara em modo _controller_.
| dev  | --flag    | Define se o sistema rodará em ambiente de desenvolvimento. Quando ativada, o sistema invocará rotinas específicas para ambientes de desenvolvimento, tais como carregar um arquivo de configuração padrão sem precisar ser colocado pelo desenvolvedor. Esta flag não é indicada para rodar em ambientes de produção já que só pode ser utilizada para facilitar ambientes de desenvolvimento.

### Arquivo de configuração

O arquivo de configuração deverá estar presente no mesmo diretório do executável em uma pasta com nome `/config/angler.cfg`. O conteúdo do arquivo será:


```properties

# Cluster configurations
cluster.authKey=abcd1234
cluster.controller.host=webhooks.my-web.services
cluster.requestTimeout=10000

# Database properties
db.deadMessages.retention=30d
db.deliveredMessages.retention=30d

# Message Processor configurations
msgproc.message_delivery_timeout=10000
msgproc.workers=500

# Configuration about the client net communication interface
net.client.protocols=restful
net.client.restful.port=80

# The default values set on retryPolicy if not set by the client
retryPolicy.defaults.interval=1d
retryPolicy.defaults.maxAttempts=7

# The limit (max or min) of interval and resend attempts
retryPolicy.limit.maxInterval=30d
retryPolicy.limit.maxAttempts=20
```
|Campo  |Descrição  |
|-------|-----------|
|cluster.authKey|Chave de autenticação utilizada no protocolo de entrada em clusters|
|cluster.controller.host|Endereço do servidor que servirá como _controller_|
|cluster.requestTimeout|O tempo limite de resposta (em milisegundos) de comunicação nos clusters. Serve tanto entre _controller_ e _broker_ quanto o inverso|
|db.deadMessages.retention|O tempo que mensagens _dead_ ficaram armazenadas no banco de logs|
|db.deliveredMessages.retention|O tempo que mensagens _delivered_ ficaram armazenadas no banco de logs|
|**msgproc.timeout***|O tempo limite de resposta (em milisegundos) de envio de mensagens para os receptores de mensagens (>=1)|
|**msgproc.workers***   |Quantos processos paralelos para envio de mensagens para os receptores estarão disponíveis na aplicação (>=1)  |
|**net.client.protocols***|Quais protocolos de comunicação serão disponibilizados para os clientes para realizar integração com o Angler. Considera-se cliente o sistema originário da mensagem. Os valores possíveis são: `restful`|
|net.client.restful.port|Qual porta será utilizada para disponibilizar o serviço de comunicação _restful_, caso o valor de `net.client.protocols` tenha-o incluído. O valor padrão é `2460`|
|retryPolicy.defaults.interval|O intervalo de tempo em que a mensagem tentará ser reenviada para o receptor. O valor desta propriedade é definido através da sintaxe de tempo do Angler. Caso o valor não seja definido, a mensagem não entrará na fila de reenvio e será descartada em caso de falha|
|retryPolicy.defaults.maxAttempts| Número inteiro que define a quantidade máxima de tentativas que o servidor fará para tentar enviar a mensagem novamente. Lembrando que, para que uma mensagem seja reenviada, obrigatóriamente será necessário incluid também a informação do `interval`. Seja informado na própria mensagem ou através da configuração `retryPolicy.defaults.interval` |
|_retryPolicy.limit_ | Diferente do _retryPolicy.defaults_ o _limit_ serve para garantir que políticas de retentativas de envio enviadas através das próprias mensagens não ultrapassem valores estabelecidos pelo servidor |
|retryPolicy.limit.maxInterval  | O valor máximo que poderá ser utilizado para definir o intervalo de retentativas |
|retryPolicy.limit.minInterval  | O valor mínimo que poderá ser utilizado para definir o intervalo de retentativas |
|retryPolicy.limit.maxAttempts  | O valor máximo que poderá ser atribuído para o campo _maxAttempts_ |

#### Configuração por variável de ambiente

Caso você não queira expor algum valor de configuração no arquivo também damos suporte a configuração através de variável de ambiente. A variável de ambiente é `ANGLER_CFG` e o valor é constituído entre a chave e valor da configuração conforme a tabela acima.

Exemplo `ANGLER_CFG=cluster.authKey='abcd1234'; retryPolicy.limit.maxInterval=30D; retryPolicy.limit.minInterval=1D`

Sintaxe:<nome_do_campo>=<valor (com ou sem ' aspas simples)>; (; ponto e vírgula para separar configurações. Espaços entre configurações opcionais)

## Sintaxe de tempo do Angler
A sintaxe de tempo do Angler é uma forma fácil para demarcar tempo. A sintaxe é constituida de um número junto a uma unidade de medida temporal, por exemplo `1D` que significa **1 dia**. Abaixo será listada as unidades de medida temporais suportadas:

- `s`: segundo;
- `m`: minuto;
- `h`: hora;
- `D`: dia
- `W`: semana;

Não é possível utilizar medidas de tempo de mês e ano pois não são medidas precisas de tempo. Por conta disso, é necessário fazer o cálculo por outra medida de tempo para atender precisamente outras medidas temporais.

# Desenvolvimento do Angler

Obrigado pelo interesse em participar da plataforma de mensageria Angler. Abaixo vamos descrever o código de conduta de desenvolvimento do projeto. Pedimos, por favor, que leia com muita atenção e siga as regras definidas no projeto.

## Esquema de Diretórios

- `/ctx` (*context*)
Aqui ficarão contidos arquivos que remetem ao estado da aplicação, tais como o valor dos arquivos de configuração, os argumentos passados para o aplicativo, o modo de execução do aplicativo (*broker* ou *controller*) .

- `/db` (*database*)
Qualquer tipo de função e procedimento relativo a persistência de dados deve ser contido neste arquivo.

- `/msgproc` (*message processor*)
Aqui será contido toda e qualquer rotina relativa ao processamento de mensagens. Para o **Angler** processamento de mensagens e todo tratamento, validação, formatação para enviar as mensagens para os destinatários.

- `/net` (*network*)
Funcionalidade de comunicação por rede ficarão contidas neste diretório. Por exemplo: comunicação para clientes ou comunicação entre nós do *cluster*.

- `/resources` (*System Companion*)
Aqui ficarão armazenados arquivos estáticos que não são arquivos fontes (.go) mas são necessário para o funcionamento.
    - `/dev` - Arquivos destinados para o ambiente de desenvolvimento;

- `/syscom` (*System Companion*)
Aqui ficarão rotinas que visam manter o funcionamento correto da plataforma Angler. Tais como: processo para limpar o banco de dados de dados antigos, serviço que reenvia mensagens com problemas de envio, etc.

- `/utils` (*utilities*)
Qualquer função ou procedimento que pode ser reutilizado em contexto geral ficarão armazenados neste diretório.


