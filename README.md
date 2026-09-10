<div align="center">

<img src="assets/images/nexa-header-logo.png" alt="Nexa Logo" width="380" />

<br />

**Controle múltiplos computadores com um único mouse e teclado através da sua rede local.**
<br />
*One keyboard. One mouse. Your devices.*

<br />
</div>

<p align="center">
  <a href="#-compatibilidade"><img src="https://img.shields.io/badge/Plataformas-Windows%2011%20%7C%20Linux%20(Wayland)-blue" alt="Plataformas"></a>
  <a href="#-desenvolvimento"><img src="https://img.shields.io/badge/Linguagem-Rust%202021-orange" alt="Linguagem"></a>
  <a href="#-licença"><img src="https://img.shields.io/badge/Licen%C3%A7a-MIT%20%2F%20Apache--2.0-green" alt="Licença"></a>
  <a href="#"><img src="https://img.shields.io/badge/Vers%C3%A3o-0.1.0-informational" alt="Versão"></a>
  <a href="#"><img src="https://img.shields.io/badge/Testes-71%20Aprovados%20(100%25)-success" alt="Status dos Testes"></a>
</p>

---

O **Nexa** é um software multiplataforma de compartilhamento de teclado, mouse e área de transferência (clipboard). Ele permite que você posicione seus computadores lado a lado e mova o cursor de uma tela para a outra simplesmente encostando na borda do monitor — exatamente como se fosse um monitor estendido, mas controlando sistemas operacionais completamente independentes.

Projetado em **Rust**, o Nexa elimina as dívidas técnicas de décadas de softwares legados, entregando criptografia de ponta a ponta, latência quase nula e compatibilidade nativa com compositores modernos (como **Wayland** no Linux e **Windows 11**).

---

## ✨ Recursos

- **Transição de Borda Fluida**: Mova o cursor suavemente entre telas de diferentes resoluções com conversão proporcional 2D automática.
- **Segurança de Ponta a Ponta**: 
  - Canal cifrado com **ChaCha20-Poly1305** e proteção estrita anti-replay.
  - Confirmação visual de pareamento via código numérico **SAS de 6 dígitos** (`XXX YYY`), prevenindo ataques de homem-no-meio (MITM).
  - Identidade soberana por chave pública **Ed25519**: nenhum computador controla o outro sem autorização explícita do usuário.
- **Conexão Direta por IP (Independente de Wi-Fi)**:
  - Funciona perfeitamente mesmo que os computadores estejam em bandas Wi-Fi diferentes (ex.: um no Wi-Fi 2.4 GHz e outro no 5 GHz) ou conectados via cabo de rede (Ethernet).
  - Suporte completo a IPv4 e arquitetura pronta para IPv6.
- **Diagnóstico Granular Integrado (`--test-connection`)**:
  - Ferramenta de teste passo a passo que diferencia se o computador está offline (*Timeout*), se a porta está bloqueada por firewall (*Connection Refused*) ou se o pareamento é pendente.
- **Descoberta de Rede Automática (Opcional)**:
  - Anúncios periódicos via UDP broadcast para encontrar aparelhos na mesma rede local com um clique.
- **Sincronização Inteligente de Clipboard (Área de Transferência)**:
  - Copie texto formatado em UTF-8 (acentos, símbolos e emojis) em uma máquina e cole na outra instantaneamente.
  - **Mecanismo Anti-Echo**: algoritmo de hash SHA-256 que bloqueia loops infinitos de cópia e teto de segurança de 10 MB contra travamentos.
- **Coalescência Adaptativa de Mouse**:
  - Compacta rajadas de movimento de mouses gamers (500 Hz – 1000 Hz) em janelas de 3 ms, reduzindo o consumo de banda em até 75% sem qualquer perda de precisão perceptível.
- **Suporte a IP Dinâmico (DHCP)**:
  - Se o roteador alterar o IP do seu computador, o Nexa atualiza o endereço com segurança sem exigir novo pareamento, pois a identidade é atrelada à chave criptográfica, e não ao endereço IP.

---

## 🖥️ Compatibilidade

| Sistema Operacional | Modos Suportados | Ambiente Gráfico / Detalhes |
| :--- | :---: | :--- |
| **Windows 11** (e Windows 10 x64) | Servidor / Cliente | Desktop interativo, suporte a Low-Level Hooks e injeção com scancodes ABNT2/US. |
| **Linux (elementary OS 8.1)** | Servidor / Cliente | Wayland nativo (Gala / Mutter) via `org.freedesktop.portal.InputCapture` e emulação `/dev/uinput`. |
| **Linux (Ubuntu / Debian x64)** | Servidor / Cliente | Pacote Debian `.deb` e serviço `systemd --user`. |

---

## 📦 Instalação

### Windows 11

#### Requisitos
- Windows 10 (64-bit) ou Windows 11.
- Permissão de usuário para executar aplicativos na sua sessão de login.

#### Passo 1: Download
Baixe o pacote executável mais recente do Nexa:
- [LINK DE DOWNLOAD DO NEXA PARA WINDOWS]

*(Caso esteja compilando do código-fonte, utilize `cargo build --release --bin nexa-daemon`)*.

#### Passo 2: Extração e Instalação
1. Extraia o arquivo baixado em uma pasta de sua preferência (exemplo: `C:\Program Files\Nexa\` ou `C:\Nexa\`).
2. A pasta conterá o executável `nexa-daemon.exe`.

#### Passo 3: Configuração de Firewall
Para permitir que o Nexa receba conexões do seu outro computador na porta padrão (`25800`), abra o **PowerShell como Administrador** e execute:
```powershell
netsh advfirewall firewall add rule name="Nexa KVM Share" dir=in action=allow protocol=TCP localport=25800
```

#### Passo 4: Inicialização Automática no Logon (Opcional)
Para que o Nexa inicie automaticamente quando você ligar o computador (com elevação de privilégios para funcionar em janelas administrativas):
```powershell
nexa-daemon.exe --autostart-enable
```

---

### Linux (elementary OS 8.1 / Ubuntu / Debian)

#### Requisitos
- Sistema operacional Linux baseado em Debian/Ubuntu de 64 bits.
- Sessão Wayland ou X11.

#### Instalação via Pacote `.deb`
1. Baixe o pacote oficial:
   - [LINK DE DOWNLOAD DO NEXA PARA LINUX (.DEB)]
2. Instale o pacote pelo terminal ou clicando duas vezes no arquivo:
   ```bash
   sudo dpkg -i nexa_0.1.0_amd64.deb
   ```
3. Garanta que o seu usuário tenha permissão para utilizar o emulador `/dev/uinput`:
   ```bash
   sudo usermod -aG input $USER
   ```
   *(Pode ser necessário encerrar a sessão e entrar novamente para atualizar o grupo)*.

#### Configuração de Firewall no Linux
Se você utiliza o **UFW**, libere a porta oficial do Nexa:
```bash
sudo ufw allow 25800/tcp comment "Nexa KVM Share"
```

#### Inicialização Automática no Linux (systemd --user)
O Nexa roda perfeitamente integrado à sua sessão gráfica do usuário:
```bash
systemctl --user enable --now nexa.service
```

---

## 🚀 Como Usar

### Cenário Típico: Controlar um Laptop Linux a partir do PC Windows

Neste exemplo, o **Computador A** (Windows 11) tem o teclado e o mouse físicos conectados, e o **Computador B** (elementary OS 8.1) fica à direita na sua mesa.

#### 1. Descobrir os Endereços IP
No computador que receberá os comandos (Cliente), abra o terminal e consulte seu IP:
```bash
nexa-daemon --interfaces
```
Exemplo de saída:
```text
=== Interfaces de Rede Locais ===
• Rede Local (Wi-Fi / Ethernet)  [Ativa]    : 192.168.1.20
```

#### 2. Iniciar o Servidor (Host)
No computador com o teclado e mouse (Windows):
```cmd
nexa-daemon.exe --server --port 25800
```

#### 3. Testar a Conexão antes de Conectar
No computador com teclado/mouse, você pode rodar um teste para validar a conectividade:
```cmd
nexa-daemon.exe --test-connection 192.168.1.20:25800
```

Se tudo estiver correto, você verá:
```text
┌────────────────────────────────────────────┐
│ Diagnóstico de conexão                     │
├────────────────────────────────────────────┤
│ Conectividade IP             ✓             │
│ Porta TCP                    ✓             │
│ Serviço Nexa                 ✓             │
│ Handshake                    ✓             │
│ Autenticação                 ✓             │
│ Resultado: Conexão validada com sucesso!   │
└────────────────────────────────────────────┘
```

#### 4. Conectar e Confirmar o Código Visual (SAS PIN)
Inicie a conexão entre os dois computadores. Na primeira vez em que eles se encontrarem, um código de segurança de 6 dígitos será mostrado nas duas telas (ex.: `482 731`):

```text
┌────────────────────────────────────────┐
│      Novo computador encontrado        │
│                                        │
│  Nome: Linux-Laptop                    │
│  Endereço: 192.168.1.20:25800          │
│                                        │
│  Código de Pareamento:                 │
│               482 731                  │
│                                        │
│        [ Aceitar ]   [ Recusar ]       │
└────────────────────────────────────────┘
```
1. Olhe para as duas telas e certifique-se de que os números coincidem.
2. Clique em **[ Aceitar ]**.
3. **Pronto!** A partir desse momento, basta mover o cursor até a borda da tela do Windows para que o mouse e o teclado comecem a controlar o Linux instantaneamente.

---

## ⚙️ Configuração

O Nexa salva suas configurações em formato declarativo `nexa.toml` na pasta do programa ou no diretório de dados do usuário:

```toml
# nexa.toml
device_name = "Meu-PC-Windows"
listen_port = 25800
edge_delay_ms = 100
auto_connect = true
log_level = "info"

[[peers]]
name = "Linux-Laptop"
ip_or_host = "192.168.1.20"
port = 25800
public_key_hex = "7f3a0981884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
is_trusted = true
alias = "Meu Linux"
manual_ip = "192.168.1.20"
manual_port = 25800
```

### Parâmetros de Linha de Comando

| Opção | Descrição |
| :--- | :--- |
| `-h, --help` | Exibe a mensagem de ajuda com todos os comandos. |
| `-v, --version` | Exibe a versão instalada do Nexa. |
| `-c, --config <ARQUIVO>` | Especifica um arquivo de configuração customizado. |
| `-p, --port <PORTA>` | Altera a porta TCP de escuta (Padrão: `25800`). |
| `--test-connection <IP[:PORT]>` | Executa a verificação diagnóstica contra um computador remoto. |
| `--interfaces` | Lista todos os adaptadores de rede e endereços IP da máquina. |
| `--no-discovery` | Desativa a descoberta por broadcast UDP na rede local. |
| `--autostart-enable` | Configura o Nexa para iniciar junto com o sistema operacional. |
| `--daemon` | Executa o processo de forma silenciosa em segundo plano. |

---

## 🔧 Solução de Problemas

### 1. Conectividade IP falha com "Timeout"
- **Sintoma**: O teste de conexão resulta em `Conectividade IP: ✗ (Timeout)`.
- **Possíveis Causas**:
  - O computador de destino está desligado ou o IP digitado está incorreto.
  - **Isolamento de Clientes (AP Isolation / Client Isolation)** ativado no roteador Wi-Fi (comum em redes de visitantes ou roteadores com isolamento entre bandas 2.4G e 5G).
- **Solução**:
  1. Verifique se o IP do outro computador não mudou usando `nexa-daemon --interfaces`.
  2. Acesse a página de administração do seu roteador Wi-Fi e desmarque a opção *AP Isolation* ou *Client Isolation*.

### 2. Porta TCP falha com "Connection Refused"
- **Sintoma**: `Conectividade IP: ✓`, mas `Porta TCP: ✗ (Connection Refused)`.
- **Possíveis Causas**:
  - O Nexa não foi iniciado no computador de destino.
  - O firewall do sistema bloqueou a porta de entrada `25800`.
- **Solução**:
  1. Certifique-se de que o `nexa-daemon` está em execução na máquina de destino.
  2. Adicione a regra de liberação da porta `25800` no Windows Firewall ou no `ufw` do Linux (conforme descrito na seção de Instalação).

### 3. Cursor não move no Linux (elementary OS / Wayland)
- **Sintoma**: O Windows indica conexão ativa, mas o cursor não se movimenta na tela do Linux.
- **Possíveis Causas**: Falta de permissão de escrita no dispositivo `/dev/uinput`.
- **Solução**:
  Adicione seu usuário ao grupo `input`:
  ```bash
  sudo usermod -aG input $USER
  ```
  Reinicie a sessão para aplicar a alteração.

---

## 🛠️ Desenvolvimento

O Nexa é desenvolvido em Rust como um workspace Cargo modular composto por 8 crates especializados:

```
nexa/
├── Cargo.toml
├── crates/
│   ├── nexa-common/       # Utilitários, logging e mascaramento de dados sensíveis
│   ├── nexa-protocol/     # Framing binário de 16 bytes e codec CRC32
│   ├── nexa-crypto/       # Criptografia Ed25519, X25519, ChaCha20-Poly1305 e SAS PIN
│   ├── nexa-net/          # Transporte assíncrono TCP, diagnóstico e descoberta LAN
│   ├── nexa-platform/     # Hooks Win32 e Portais Wayland Linux
│   ├── nexa-core/         # SessionEngine, Topologia 2D, FSM e Anti-Echo Clipboard
│   ├── nexa-ui/           # Estado reativo da UI, canvas de monitores e telas
│   └── nexa-daemon/       # Binário executável e daemon de sistema
└── scripts/               # Scripts de empacotamento para Windows e Debian (.deb)
```

### Compilação e Testes
```bash
# Clonar o repositório
git clone https://github.com/rafaelmaciels/Nexa.git
cd Nexa

# Executar a suíte de 71 testes automatizados
cargo test --all

# Compilar em modo release
cargo build --release
```

---

## 🤝 Contribuição

Contribuições são muito bem-vindas! Para contribuir:

1. Faça um **Fork** do repositório.
2. Crie uma branch para sua funcionalidade (`git checkout -b feature/minha-melhoria`).
3. Faça o commit das suas alterações (`git commit -m "Adiciona suporte a nova funcionalidade"`).
4. Envie para o GitHub (`git push origin feature/minha-melhoria`).
5. Abra um **Pull Request**.

---

## 📄 Licença

Distribuído sob licença dupla **MIT** ou **Apache-2.0**. Consulte o arquivo [LICENSE](LICENSE) para mais detalhes.

---

## 📞 Suporte

- **Problemas e Sugestões**: Abra uma issue no [GitHub Issues](https://github.com/rafaelmaciels/Nexa/issues).
- **Discussões e Dúvidas**: Participe do [GitHub Discussions](https://github.com/rafaelmaciels/Nexa/discussions).

---

## ⭐ Apoie o Projeto

Se o **Nexa** é útil no seu dia a dia, considere deixar uma **estrela (Star ⭐)** no topo do repositório! Isso ajuda o projeto a crescer e alcançar mais pessoas.
