# 🐧 Manual de Instalação e Operação — Linux (elementary OS 8.1 Wayland / Ubuntu / Debian)

Guia passo a passo completo para instalação, permissões de dispositivo `/dev/uinput`, liberação de firewall, execução e inicialização automática via `systemd --user` no Linux.

---

## 📋 Índice
1. [Requisitos do Sistema](#-requisitos-do-sistema)
2. [Instalação via Pacote Oficial (`.deb`)](#-instalação-via-pacote-oficial-deb)
3. [Permissões do Emulador de Entrada (`/dev/uinput`)](#-permissões-do-emulador-de-entrada-devuinput)
4. [Configuração do Firewall (UFW)](#-configuração-do-firewall-ufw)
5. [Como Usar (Passo a Passo)](#-como-usar-passo-a-passo)
   - [Cenário 1: O Linux será controlado pelo PC com mouse/teclado (Modo Cliente / Guest)](#cenário-1-o-linux-será-controlado-pelo-pc-com-mouseteclado-modo-cliente--guest)
   - [Cenário 2: O Linux tem o mouse/teclado físico (Modo Servidor / Host)](#cenário-2-o-linux-tem-o-mouseteclado-físico-modo-servidor--host)
6. [Diagnóstico de Rede Integrado](#-diagnóstico-de-rede-integrado)
7. [Inicialização Automática via `systemd --user`](#-inicialização-automática-via-systemd---user)
8. [Arquivo de Configuração (`nexa.toml`)](#-arquivo-de-configuração-nexatoml)
9. [Solução de Problemas Frequentes](#-solução-de-problemas-frequentes)

---

## 💻 Requisitos do Sistema

* **Sistema Operacional**:
  * **elementary OS 8.1** (ambiente gráfico Gala / Wayland)
  * **Ubuntu 22.04 LTS / 24.04 LTS** ou **Debian 12 Bookworm** (64-bit)
* **Arquitetura**: AMD64 (x86_64).
* **Subsistema de Entrada**: Módulo de kernel `uinput` ativo (padrão em quase todas as distribuições modernas).

---

## 📦 Instalação via Pacote Oficial (`.deb`)

### Opção A: Instalando o pacote `.deb` pré-compilado
1. Baixe o pacote oficial mais recente:
   - 📦 [**Download nexa_0.1.0_amd64.deb (GitHub Releases)**](https://github.com/rafaelmaciels/Nexa/releases/latest/download/nexa_0.1.0_amd64.deb)
2. Abra o terminal na pasta do arquivo baixado e instale:
   ```bash
   sudo dpkg -i nexa_0.1.0_amd64.deb
   ```
3. Caso falte alguma dependência do sistema, corrija com:
   ```bash
   sudo apt-get install -f
   ```

O executável será instalado globalmente em `/usr/bin/nexa-daemon`.

### Opção B: Compilando do código-fonte
Se você é desenvolvedor e deseja compilar o Nexa diretamente do repositório:
```bash
# Instala dependências básicas de compilação
sudo apt update && sudo apt install -y build-essential pkg-config libssl-dev

# Compila o binário em modo release otimizado
cargo build --release --bin nexa-daemon

# Copia para os binários do sistema
sudo cp target/release/nexa-daemon /usr/bin/nexa-daemon
```

---

## 🔐 Permissões do Emulador de Entrada (`/dev/uinput`)

Para que o Nexa consiga movimentar o cursor do mouse e simular teclas no elementary OS 8.1 (Wayland) e Ubuntu sem exigir que o programa rode como `root`, execute o script automático de 1 linha:

```bash
curl -sSL https://raw.githubusercontent.com/rafaelmaciels/Nexa/main/scripts/setup-linux-uinput.sh | bash
```

### Opcional: Configuração Manual de Permissões
Se preferir executar os comandos manualmente passo a passo:
```bash
# 1. Carrega o módulo uinput no kernel
sudo modprobe uinput

# 2. Cria a regra udev definitiva para liberar acesso ao cursor
echo 'KERNEL=="uinput", SUBSYSTEM=="misc", TAG+="uaccess", OPTIONS+="static_node=uinput", MODE="0666"' | sudo tee /etc/udev/rules.d/99-nexa-uinput.rules

# 3. Adiciona seu usuário ao grupo input
sudo usermod -aG input $USER

# 4. Aplica imediatamente sem precisar reiniciar ou deslogar
sudo udevadm control --reload-rules
sudo udevadm trigger --name-match=uinput || sudo udevadm trigger
sudo chmod 666 /dev/uinput
```

Para verificar se o seu usuário já possui acesso:
```bash
groups | grep input
```
Se a palavra `input` for exibida, o sistema está 100% pronto!

---

## 🛡️ Configuração do Firewall (UFW)

Se você utiliza o firewall padrão do elementary OS / Ubuntu (**UFW**), libere a porta TCP do Nexa:

```bash
sudo ufw allow 25800/tcp comment "Nexa KVM Share"
```

Se desejar habilitar também a descoberta automática na rede local:
```bash
sudo ufw allow 25801/udp comment "Nexa Discovery UDP"
```

---

## 🚀 Como Usar (Passo a Passo)

### Cenário 1: O Linux será controlado pelo PC com mouse/teclado (Modo Cliente / Guest)

Este é o cenário mais comum: seu laptop com elementary OS 8.1 fica à direita da mesa e você quer controlá-lo usando o mouse e teclado do seu PC principal.

1. No terminal do Linux, consulte seu IP local:
   ```bash
   nexa-daemon --interfaces
   ```
   Exemplo de saída: `192.168.1.20`.

2. Conecte ao seu computador principal (Host):
   ```bash
   nexa-daemon --client 192.168.1.15:25800
   ```
   *(Substitua `192.168.1.15` pelo IP do computador com o teclado e mouse)*.

3. O terminal mostrará o **Código de Pareamento de Segurança (SAS PIN)**:
   ```text
   ┌──────────────────────────────────────────────────┐
   │          CONEXÃO ESTABELECIDA COM SUCESSO        │
   ├──────────────────────────────────────────────────┤
   │  Código de Pareamento de Segurança (SAS PIN):   │
   │                     482 731                      │
   └──────────────────────────────────────────────────┘
   ```
4. Confirme que os números batem com a tela do outro computador.
5. **Pronto!** Agora, quando o cursor atravessar a borda da tela do host, ele entrará suavemente na tela do elementary OS, permitindo digitar, mover janelas e colar textos normalmente.

---

### Cenário 2: O Linux tem o mouse/teclado físico (Modo Servidor / Host)

Se o mouse e teclado físicos estão conectados no Linux:

1. Inicie o Nexa em modo servidor:
   ```bash
   nexa-daemon --server --port 25800
   ```
2. O Nexa ficará ouvindo na porta `25800` e imprimirá o código SAS assim que o outro computador se conectar.
3. Arraste o cursor até a borda da tela para assumir o controle da outra máquina.

* **Atalho de Retorno de Emergência**: Pressione a tecla `[ Scroll Lock ]` a qualquer instante para trazer o controle de volta ao Linux.

---

## 🔍 Diagnóstico de Rede Integrado

Para checar se a máquina de destino está respondendo e se as portas e firewalls estão abertos:

```bash
nexa-daemon --test-connection 192.168.1.15:25800
```

Se tudo estiver correto, todos os 5 itens receberão o sinal `✓` (Conectividade IP, Porta TCP, Serviço Nexa, Handshake e Autenticação).

---

## ⚙️ Inicialização Automática via `systemd --user`

O Nexa é projetado para rodar como um serviço integrado à sessão gráfica do seu usuário, iniciando silenciosamente em segundo plano no login.

### 1. Criar o Arquivo de Serviço do Usuário
Crie a pasta de serviços do seu usuário caso não exista:
```bash
mkdir -p ~/.config/systemd/user
```

Gere a unidade `nexa.service`:
```bash
cat <<EOF > ~/.config/systemd/user/nexa.service
[Unit]
Description=Nexa KVM Share Daemon
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart=/usr/bin/nexa-daemon --daemon
Restart=always
RestartSec=3

[Install]
WantedBy=graphical-session.target
EOF
```

### 2. Ativar e Iniciar o Serviço
```bash
systemctl --user daemon-reload
systemctl --user enable --now nexa.service
```

### 3. Verificar o Status do Serviço
```bash
systemctl --user status nexa.service
```

Para desativar no futuro:
```bash
systemctl --user disable --now nexa.service
```

---

## 📄 Arquivo de Configuração (`nexa.toml`)

No Linux, você pode criar o arquivo de configuração declarativo em `~/.config/nexa/nexa.toml` ou no diretório atual:

```toml
# nexa.toml
device_name = "Elementary-Laptop"
listen_port = 25800
edge_delay_ms = 100
auto_connect = true
log_level = "info"

[[peers]]
name = "Meu-PC-Windows"
ip_or_host = "192.168.1.15"
port = 25800
is_trusted = true
```

---

## ❓ Solução de Problemas Frequentes

### 1. "Permission Denied" ao abrir `/dev/uinput`
* **Sintoma**: O log exibe erro de permissão ao criar o dispositivo de entrada virtual.
* **Causa**: Seu usuário ainda não pertence ao grupo `input` ou a regra udev não foi lida.
* **Solução**:
  1. Verifique com `ls -l /dev/uinput` (deve pertencer ao grupo `input` com permissões `crw-rw----`).
  2. Adicione seu usuário com `sudo usermod -aG input $USER`.
  3. Encerre a sessão gráfica (logout) e entre novamente.

### 2. Sessão Wayland não recebe cliques
* Verifique se o pacote `systemd` e os módulos de kernel estão ativos:
  ```bash
  sudo modprobe uinput
  ```
* Para carregar o módulo `uinput` automaticamente no boot:
  ```bash
  echo "uinput" | sudo tee /etc/modules-load.d/uinput.conf
  ```

### 3. Conexão rejeitada com `Connection Refused`
* O firewall pode estar ativo sem a porta liberada. Execute:
  ```bash
  sudo ufw allow 25800/tcp
  ```
