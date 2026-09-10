# 🪟 Manual de Instalação e Operação — Windows 11 / Windows 10 (x64)

Guia passo a passo completo para instalação, configuração de firewall, execução e inicialização automática do **Nexa** no Windows.

---

## 📋 Índice
1. [Requisitos do Sistema](#-requisitos-do-sistema)
2. [Instalação e Obtenção do Executável](#-instalação-e-obtenção-do-executável)
3. [Configuração do Firewall do Windows](#-configuração-do-firewall-do-windows)
4. [Como Usar (Passo a Passo)](#-como-usar-passo-a-passo)
   - [Cenário 1: Este computador tem o mouse e teclado (Modo Servidor / Host)](#cenário-1-este-computador-tem-o-mouse-e-teclado-modo-servidor--host)
   - [Cenário 2: Este computador será controlado à distância (Modo Cliente / Guest)](#cenário-2-este-computador-será-controlado-à-distância-modo-cliente--guest)
5. [Diagnóstico de Rede Integrado](#-diagnóstico-de-rede-integrado)
6. [Atalhos de Emergência e Comportamento de Borda](#-atalhos-de-emergência-e-comportamento-de-borda)
7. [Inicialização Automática no Logon (UAC Elevado)](#-inicialização-automática-no-logon-uac-elevado)
8. [Arquivo de Configuração (`nexa.toml`)](#-arquivo-de-configuração-nexatoml)
9. [Solução de Problemas Frequentes](#-solução-de-problemas-frequentes)

---

## 💻 Requisitos do Sistema

* **Sistema Operacional**: Windows 11 (todas as edições) ou Windows 10 (64-bit, versão 1809 ou superior).
* **Permissões**: Acesso de Administrador local para criar a regra no Firewall do Windows e configurar a tarefa de logon elevado.
* **Rede**: Conexão Wi-Fi ou cabo de rede Ethernet na mesma rede local que o outro computador (computadores em bandas 2.4 GHz e 5.0 GHz funcionam normalmente).

---

## 📦 Instalação e Obtenção do Executável

### Opção A: Download do Pacote Oficial (.ZIP)
1. Baixe o arquivo mais recente:
   - 💾 [**Download nexa-windows-x64.zip (GitHub Releases)**](https://github.com/rafaelmaciels/Nexa/releases/latest/download/nexa-windows-x64.zip)
2. Extraia o conteúdo empacotado em uma pasta permanente, recomendada:
   ```text
   C:\Nexa\
   ```
3. A pasta conterá o executável `nexa-daemon.exe`, `install-autostart.bat` e a configuração padrão.

### Opção B: Compilando do código-fonte (para desenvolvedores)
Se você clonou o repositório Nexa e possui o Rust instalado:
```powershell
cd nexa
cargo build --release --bin nexa-daemon
```
O executável compilado e otimizado estará em `target\release\nexa-daemon.exe`.

---

## 🛡️ Configuração do Firewall do Windows

Para permitir que o computador receba comandos de rede na porta oficial do Nexa (`25800`), abra o **PowerShell como Administrador** e execute:

```powershell
netsh advfirewall firewall add rule name="Nexa KVM Share" dir=in action=allow protocol=TCP localport=25800 profile=any
```

> **Nota**: Se você também desejar utilizar a descoberta automática de computadores por anúncio na rede local, libere a porta UDP de broadcast:
> ```powershell
> netsh advfirewall firewall add rule name="Nexa Discovery UDP" dir=in action=allow protocol=UDP localport=25801 profile=any
> ```

---

## 🚀 Como Usar (Passo a Passo)

### 🖥️ Painel Gráfico de Controle (GUI) — Recomendado
Basta dar dois cliques no executável `nexa-daemon.exe` (ou executar `.\nexa-daemon.exe` no PowerShell):
* O navegador padrão abrirá imediatamente na interface gráfica do Nexa (`http://127.0.0.1:25802`).
* Permite posicionar visualmente as telas (Esquerda / Direita), monitorar conexões e realizar diagnósticos de rede em 1 clique.

---

### Cenário 1: Este computador tem o mouse e teclado (Modo Servidor / Host)

Se o mouse e o teclado físicos estão conectados neste computador Windows e você deseja controlar um outro computador (por exemplo, um notebook ao lado):

1. Descubra o IP desta máquina executando:
   ```powershell
   .\nexa-daemon.exe --interfaces
   ```
   Anote o IP exibido na interface de rede ativa (exemplo: `192.168.1.15`).

2. Inicie o Nexa em modo servidor:
   ```powershell
   .\nexa-daemon.exe --server --port 25800
   ```

3. O Nexa ficará aguardando a conexão do outro computador.
4. Quando o outro computador se conectar, um **Código de Pareamento de Segurança (SAS PIN de 6 dígitos)** será exibido no terminal:
   ```text
   ┌──────────────────────────────────────────────────┐
   │           NOVO COMPUTADOR CONECTADO              │
   ├──────────────────────────────────────────────────┤
   │  Código de Pareamento de Segurança (SAS PIN):   │
   │                     482 731                      │
   └──────────────────────────────────────────────────┘
   ```
5. Olhe para a tela do outro computador e confirme se o número é o mesmo.
6. **Pronto!** Basta mover o mouse para a borda direita do seu monitor e o cursor entrará no outro computador.

---

### Cenário 2: Este computador será controlado à distância (Modo Cliente / Guest)

Se este computador Windows fica ao lado da sua mesa e será controlado pelo teclado/mouse do seu outro computador (Host):

1. Inicie o Nexa conectando diretamente ao IP do servidor:
   ```powershell
   .\nexa-daemon.exe --client 192.168.1.15:25800
   ```
   *(Substitua `192.168.1.15` pelo IP do computador com o teclado/mouse)*.

2. O Nexa efetuará o handshake criptográfico ChaCha20-Poly1305 e mostrará o mesmo PIN SAS de 6 dígitos.
3. Assim que o Host empurrar o cursor até a borda da tela dele, seu Windows passará a responder aos movimentos, cliques, digitação e você poderá copiar e colar textos entre as duas máquinas!

---

## 🔍 Diagnóstico de Rede Integrado

Caso você enfrente qualquer dificuldade de conexão entre as duas máquinas, use a ferramenta de diagnóstico integrada:

```powershell
.\nexa-daemon.exe --test-connection 192.168.1.20:25800
```

O Nexa executará 5 testes diagnósticos instantâneos:
```text
┌────────────────────────────────────────────┐
│ Diagnóstico de conexão                     │
├────────────────────────────────────────────┤
│ Destino                                    │
│ 192.168.1.20:25800                         │
│                                            │
│ Conectividade IP             ✓             │
│ Porta TCP                    ✓             │
│ Serviço Nexa                 ✓             │
│ Handshake                    ✓             │
│ Autenticação                 ✓             │
│                                            │
│ Resultado                                  │
│ ✓ Conexão validada com sucesso!            │
└────────────────────────────────────────────┘
```

Se algum item falhar, o Nexa exibirá a causa provável (ex: *Porta bloqueada pelo Firewall*, *Isolamento de clientes no roteador* ou *Máquina de destino desligada*).

---

## ⌨️ Atalhos de Emergência e Comportamento de Borda

* **Atalhos de Emergência ([ Esc ] ou [ Scroll Lock ])**:
  Se o cursor estiver capturado no computador remoto ou se você precisar retornar o foco imediatamente para o computador local (Host), basta pressionar qualquer uma das teclas:
  ```text
  [ Esc ]   ou   [ Scroll Lock ]
  ```
  O Nexa quebra a captura instantaneamente e devolve o controle ao mouse e teclado locais.

* **Transição Fluida e Margem de Tolerância (Zero Travamentos)**:
  - Ao encostar o cursor na borda mapeada, o Nexa aciona a transição imediata sem atrito.
  - Em ambientes com múltiplos monitores, o Nexa detecta os limites reais de cada tela (inclusive coordenadas com offsets negativos), mantendo a movimentação contínua e impedindo que o cursor fique preso nas quinas físicas do Windows.
  - Durante o controle remoto, o cursor físico local permanece centralizado dinamicamente em segundo plano, garantindo que o sistema operacional local continue respondendo sem congelamento ou atrasos de entrada.

* **Sincronização de Clipboard (Área de Transferência)**:
  Copie normalmente (`Ctrl + C`) em uma tela, atravesse com o mouse para a outra máquina e cole (`Ctrl + V`). O Nexa utiliza codificação UTF-8 completa com suporte a acentos, pontuação em português (ABNT2) e emojis, com proteção anti-echo via hash SHA-256.

---

## ⚙️ Inicialização Automática no Logon (UAC Elevado)

Para que o mouse continue funcionando mesmo sobre janelas executadas como Administrador ou caixas de diálogo do UAC (Controle de Conta de Usuário), o Nexa deve iniciar pelo Agendador de Tarefas do Windows com privilégio elevado.

Para gerar e instalar a tarefa automaticamente:

1. Abra o **PowerShell como Administrador**.
2. Execute o comando:
   ```powershell
   schtasks /create /tn "NexaDaemon" /tr "\"C:\Nexa\nexa-daemon.exe\" --daemon" /sc onlogon /rl highest /f
   ```
3. O Nexa agora iniciará silenciosamente em segundo plano toda vez que você entrar no Windows.

Para remover a inicialização automática no futuro:
```powershell
schtasks /delete /tn "NexaDaemon" /f
```

---

## 📄 Arquivo de Configuração (`nexa.toml`)

Você pode criar um arquivo `nexa.toml` na mesma pasta do executável para definir configurações fixas:

```toml
# nexa.toml
device_name = "Meu-PC-Windows"
listen_port = 25800
edge_delay_ms = 100
auto_connect = true
log_level = "info"

[[peers]]
name = "Notebook-Linux"
ip_or_host = "192.168.1.20"
port = 25800
is_trusted = true
```

---

## ❓ Solução de Problemas Frequentes

### 1. O teste de conexão resulta em `Conectividade IP: ✗ (Timeout)`
* Verifique se ambos os computadores estão conectados à rede local.
* Se um computador estiver no Wi-Fi 2.4 GHz e outro no Wi-Fi 5 GHz, acesse as configurações do roteador e desative a opção **AP Isolation** ou **Isolamento de Clientes**.

### 2. O teste resulta em `Porta TCP: ✗ (Connection Refused)`
* Certifique-se de que o Nexa está aberto no outro computador (`--server`).
* Revise a regra do Firewall do Windows conforme a seção [Configuração do Firewall](#-configuração-do-firewall-do-windows).

### 3. O cursor não responde quando clico em janelas de Administrador (Prompt de Comando / Gerenciador de Tarefas)
* No Windows, aplicativos sem privilégios elevados não podem injetar eventos em aplicativos elevados.
* Instale o Nexa utilizando a tarefa do Agendador com privilégios máximos (`/rl highest`), conforme explicado na seção [Inicialização Automática](#-inicialização-automática-no-logon-uac-elevado).
