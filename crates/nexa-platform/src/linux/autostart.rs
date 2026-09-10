/// Gerenciador de Inicialização Automática para Linux (elementary OS 8.1 / systemd --user)
pub struct LinuxAutostartManager;

impl LinuxAutostartManager {
    /// Gera o conteúdo do serviço systemd user (~/.config/systemd/user/nexa.service)
    pub fn generate_systemd_user_service(exe_path: &str) -> String {
        format!(
            r#"[Unit]
Description=Nexa KVM Share Daemon
Documentation=https://github.com/nexa-share/nexa
After=graphical-session.target
PartOf=graphical-session.target

[Service]
Type=simple
ExecStart={} --daemon
Restart=always
RestartSec=3
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=graphical-session.target
"#,
            exe_path
        )
    }

    /// Gera o arquivo .desktop para autostart XDG (~/.config/autostart/nexa.desktop)
    pub fn generate_desktop_entry(exe_path: &str) -> String {
        format!(
            r#"[Desktop Entry]
Type=Application
Name=Nexa Daemon
Comment=Compartilhamento de mouse e teclado de baixa latência
Exec={} --daemon
Icon=preferences-desktop-display
Terminal=false
Categories=Utility;System;
StartupNotify=false
X-GNOME-Autostart-enabled=true
"#,
            exe_path
        )
    }
}
