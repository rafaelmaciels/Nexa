#!/usr/bin/env bash
# Script de Construção do Pacote .deb para elementary OS 8.1 (Gala / Wayland)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
PKG_VERSION="0.1.0"
PKG_NAME="nexa_${PKG_VERSION}_amd64"
DIST_DIR="$PROJECT_ROOT/dist/$PKG_NAME"

echo "=== Empacotando Nexa para elementary OS 8.1 (.deb) ==="

# 1. Compilação Release
echo "-> Compilando release..."
cd "$PROJECT_ROOT"
cargo build --release --bin nexa-daemon

# 2. Estrutura de Diretórios Debian
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR/DEBIAN"
mkdir -p "$DIST_DIR/usr/bin"
mkdir -p "$DIST_DIR/usr/share/applications"
mkdir -p "$DIST_DIR/usr/lib/systemd/user"
mkdir -p "$DIST_DIR/etc/udev/rules.d"

# 3. Copia Binário
cp "$PROJECT_ROOT/target/release/nexa-daemon" "$DIST_DIR/usr/bin/nexa-daemon"
chmod 755 "$DIST_DIR/usr/bin/nexa-daemon"

# 4. Cria arquivo de controle do pacote
cat <<EOF > "$DIST_DIR/DEBIAN/control"
Package: nexa
Version: ${PKG_VERSION}
Section: utils
Priority: optional
Architecture: amd64
Depends: libc6, systemd
Maintainer: Nexa Contributors <contributors@nexa-share.org>
Description: Compartilhamento de teclado, mouse e clipboard ultrarrápido
 Nexa é uma alternativa moderna, segura e com criptografia de ponta a ponta
 ao Deskflow/Synergy, projetada especificamente para Wayland e Windows 11.
EOF

# 5. Cria Regra udev para acesso ao /dev/uinput sem sudo
cat <<EOF > "$DIST_DIR/etc/udev/rules.d/99-nexa-uinput.rules"
KERNEL=="uinput", GROUP="input", MODE="0660"
EOF

# 6. Cria Unidade systemd --user
cat <<EOF > "$DIST_DIR/usr/lib/systemd/user/nexa.service"
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

# 7. Cria Entrada de Desktop XDG
cat <<EOF > "$DIST_DIR/usr/share/applications/io.elementary.nexa.desktop"
[Desktop Entry]
Type=Application
Name=Nexa
Comment=Compartilhamento de mouse e teclado
Exec=nexa-daemon
Icon=preferences-desktop-display
Terminal=false
Categories=Utility;System;
EOF

# 8. Script postinst para configurar udev e permissões
cat <<EOF > "$DIST_DIR/DEBIAN/postinst"
#!/bin/sh
set -e
udevadm control --reload-rules || true
udevadm trigger || true
echo "Nexa instalado com sucesso no elementary OS 8.1."
exit 0
EOF
chmod 755 "$DIST_DIR/DEBIAN/postinst"

# 9. Constrói o pacote .deb usando dpkg-deb se disponível
if command -v dpkg-deb >/dev/null 2>&1; then
    dpkg-deb --build "$DIST_DIR" "$PROJECT_ROOT/dist/${PKG_NAME}.deb"
    echo "=== Pacote gerado com sucesso: dist/${PKG_NAME}.deb ==="
else
    echo "dpkg-deb não encontrado no ambiente atual; estrutura preparada em: $DIST_DIR"
fi
