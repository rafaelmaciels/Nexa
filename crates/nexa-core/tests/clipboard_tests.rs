use std::sync::Mutex;
use nexa_common::NexaError;
use nexa_core::ClipboardSyncEngine;
use nexa_platform::ClipboardManager;
use nexa_protocol::NexaPacket;

/// Mock de ClipboardManager com armazenamento seguro em memória
struct MockClipboard {
    content: Mutex<Option<String>>,
}

impl MockClipboard {
    fn new(initial: Option<&str>) -> Self {
        Self {
            content: Mutex::new(initial.map(|s| s.to_string())),
        }
    }
}

impl ClipboardManager for MockClipboard {
    fn get_text(&self) -> Result<Option<String>, NexaError> {
        Ok(self.content.lock().unwrap().clone())
    }

    fn set_text(&self, text: &str) -> Result<(), NexaError> {
        *self.content.lock().unwrap() = Some(text.to_string());
        Ok(())
    }
}

#[test]
fn test_clipboard_anti_echo_suppression() {
    let mut engine = ClipboardSyncEngine::default();
    let mock = MockClipboard::new(None);

    let remote_text = "https://github.com/nexa-share/nexa";

    // 1. Recebe payload remoto e injeta no clipboard local
    let updated = engine
        .on_remote_clipboard_received(remote_text, &mock)
        .expect("Falha ao processar clipboard remoto");
    assert!(updated, "O clipboard local deveria ter sido atualizado");
    assert_eq!(mock.get_text().unwrap().as_deref(), Some(remote_text));

    // 2. Simula o listener do SO local detectando a nova alteração no clipboard
    let echo_packet = engine.on_local_clipboard_changed(remote_text);

    // DEVE ser suprimido (Anti-Echo ativo: não reenviar o que acabou de ser injetado)
    assert!(
        echo_packet.is_none(),
        "Eco de clipboard NÃO foi suprimido! Risco iminente de loop infinito."
    );
}

#[test]
fn test_clipboard_idempotent_duplicate() {
    let mut engine = ClipboardSyncEngine::default();
    let text = "Copiar várias vezes com Ctrl+C seguidos";

    // Primeira cópia: deve emitir pacote
    let p1 = engine.on_local_clipboard_changed(text);
    assert!(p1.is_some(), "Primeira cópia deveria gerar pacote");

    // Segunda cópia do mesmo texto: deve ser ignorada
    let p2 = engine.on_local_clipboard_changed(text);
    assert!(p2.is_none(), "Cópia duplicada deveria ser ignorada");

    // Terceira cópia com texto diferente: deve emitir pacote novo
    let p3 = engine.on_local_clipboard_changed("Novo Texto Copiado");
    assert!(p3.is_some(), "Novo texto deveria gerar pacote");
}

#[test]
fn test_clipboard_payload_size_limit() {
    // Configura limite de 100 KB para teste
    let mut engine = ClipboardSyncEngine::new(100 * 1024);
    let mock = MockClipboard::new(None);

    let huge_text = "A".repeat(150 * 1024); // 150 KB > 100 KB limite

    // Local
    let local_res = engine.on_local_clipboard_changed(&huge_text);
    assert!(
        local_res.is_none(),
        "Payload excessivo local deveria ter sido descartado"
    );

    // Remoto
    let remote_res = engine
        .on_remote_clipboard_received(&huge_text, &mock)
        .expect("Erro ao processar");
    assert!(
        !remote_res,
        "Payload excessivo remoto deveria ter sido rejeitado sem aplicar no SO"
    );
    assert!(mock.get_text().unwrap().is_none());
}

#[test]
fn test_bidirectional_clipboard_sync_simulation() {
    // Nó 1: Windows 11
    let mut node_win = ClipboardSyncEngine::default();
    let clip_win = MockClipboard::new(None);

    // Nó 2: elementary OS 8.1
    let mut node_linux = ClipboardSyncEngine::default();
    let clip_linux = MockClipboard::new(None);

    // === PASSO 1: Usuário copia no Windows ===
    let text_win = "Texto originado no Windows 11 com acentuação e Ç!";
    clip_win.set_text(text_win).unwrap();

    let packet_from_win = node_win
        .on_local_clipboard_changed(text_win)
        .expect("Windows deveria emitir pacote");

    // Transmissão de rede para Linux
    if let NexaPacket::Clipboard(data) = packet_from_win {
        let applied = node_linux
            .on_remote_clipboard_received(&data.text, &clip_linux)
            .unwrap();
        assert!(applied, "Linux deveria ter aplicado o clipboard");
    } else {
        panic!("Tipo de pacote incorreto");
    }

    // Verifica que Linux recebeu
    assert_eq!(clip_linux.get_text().unwrap().as_deref(), Some(text_win));

    // Monitor do Linux detecta a alteração e tenta reenviar:
    let linux_echo = node_linux.on_local_clipboard_changed(text_win);
    assert!(linux_echo.is_none(), "Linux NÃO deve ecoar de volta para o Windows!");

    // === PASSO 2: Agora usuário copia no Linux ===
    let text_linux = "Resposta originada no elementary OS 8.1: Gala Wayland 🚀";
    clip_linux.set_text(text_linux).unwrap();

    let packet_from_linux = node_linux
        .on_local_clipboard_changed(text_linux)
        .expect("Linux deveria emitir pacote");

    // Transmissão de rede para Windows
    if let NexaPacket::Clipboard(data) = packet_from_linux {
        let applied = node_win
            .on_remote_clipboard_received(&data.text, &clip_win)
            .unwrap();
        assert!(applied, "Windows deveria ter aplicado o clipboard");
    } else {
        panic!("Tipo de pacote incorreto");
    }

    // Verifica que Windows recebeu
    assert_eq!(clip_win.get_text().unwrap().as_deref(), Some(text_linux));

    // Monitor do Windows detecta a alteração e tenta reenviar:
    let win_echo = node_win.on_local_clipboard_changed(text_linux);
    assert!(win_echo.is_none(), "Windows NÃO deve ecoar de volta para o Linux!");
}

#[test]
fn test_sync_on_screen_switch() {
    let mut engine = ClipboardSyncEngine::default();
    let mock = MockClipboard::new(Some("Texto já pronto antes da transição"));

    // Na transição de tela:
    let packet = engine
        .sync_on_screen_switch(&mock)
        .expect("Erro ao sincronizar na transição")
        .expect("Deveria emitir pacote pendente");

    if let NexaPacket::Clipboard(data) = packet {
        assert_eq!(data.text, "Texto já pronto antes da transição");
    } else {
        panic!("Tipo de pacote inválido");
    }

    // Segunda transição sem alteração no clipboard: não deve enviar nada
    let packet_none = engine
        .sync_on_screen_switch(&mock)
        .expect("Erro na segunda sincronização");
    assert!(packet_none.is_none());
}

#[test]
fn test_unicode_and_emojis() {
    let mut sender_engine = ClipboardSyncEngine::default();
    let mut receiver_engine = ClipboardSyncEngine::default();
    let mock = MockClipboard::new(None);

    let complex_text = "🇧🇷 Padrão ABNT2: maçã, ação, vôo, frequência | 日本語 | 🚀💻🔥";

    let packet = sender_engine
        .on_local_clipboard_changed(complex_text)
        .expect("Deveria suportar unicode complexo");

    if let NexaPacket::Clipboard(data) = packet {
        assert_eq!(data.text, complex_text);

        // Aplica no mock remoto do nó de destino
        let applied = receiver_engine
            .on_remote_clipboard_received(&data.text, &mock)
            .unwrap();
        assert!(applied);
        assert_eq!(mock.get_text().unwrap().as_deref(), Some(complex_text));
    }
}
