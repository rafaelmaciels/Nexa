#!/usr/bin/env bash
# Script de Configuração Instantânea de Permissões para o Nexa no Linux
# Permite criar mouses e teclados virtuais sem precisar rodar com sudo.

set -euo pipefail

echo "=== Configurando Permissões do Nexa no Linux (/dev/uinput) ==="

# 1. Carrega o módulo uinput no kernel se não estiver carregado
sudo modprobe uinput || true

# 2. Instala a regra udev com TAG+="uaccess" e MODE="0666" para acesso imediato
RULE_FILE="/etc/udev/rules.d/99-nexa-uinput.rules"
echo "-> Gravando regra em $RULE_FILE..."
sudo bash -c "cat <<EOF > $RULE_FILE
KERNEL==\"uinput\", SUBSYSTEM==\"misc\", TAG+=\"uaccess\", OPTIONS+=\"static_node=uinput\", MODE=\"0666\"
EOF"

# 3. Adiciona o usuário atual ao grupo input por garantia
sudo usermod -aG input "$USER" || true

# 4. Recarrega as regras do udev e aplica imediatamente
echo "-> Recarregando regras udev..."
sudo udevadm control --reload-rules || true
sudo udevadm trigger --name-match=uinput || sudo udevadm trigger || true

# 5. Garante permissão no nó atual se já existir
if [ -e /dev/uinput ]; then
    sudo chmod 666 /dev/uinput
fi

echo "=============================================================="
echo "✓ Permissões configuradas com sucesso!"
echo "Agora você pode rodar o Nexa normalmente com:"
echo "   nexa-daemon --client IP_DO_WINDOWS:25800"
echo "=============================================================="
