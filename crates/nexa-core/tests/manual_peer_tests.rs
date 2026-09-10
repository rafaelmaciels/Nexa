use tempfile::tempdir;
use nexa_core::{AppConfig, PeerConfig, DEFAULT_PORT};

#[test]
fn test_manual_peer_addition_and_persistence() {
    let tmp = tempdir().expect("Falha ao criar tempdir");
    let cfg_path = tmp.path().join("manual_nexa.toml");

    let mut cfg = AppConfig::default();
    assert_eq!(cfg.listen_port, DEFAULT_PORT);
    assert_eq!(cfg.listen_port, 25800);

    // Adiciona computador manualmente: 192.168.1.20:25800 com nome "Meu Linux"
    cfg.add_or_update_manual_peer("192.168.1.20", 25800, Some("Meu Linux"), None);

    assert_eq!(cfg.peers.len(), 1);
    assert_eq!(cfg.peers[0].name, "Meu Linux");
    assert_eq!(cfg.peers[0].ip_or_host, "192.168.1.20");
    assert_eq!(cfg.peers[0].port, 25800);
    assert_eq!(cfg.peers[0].alias.as_deref(), Some("Meu Linux"));
    assert_eq!(cfg.peers[0].manual_ip.as_deref(), Some("192.168.1.20"));
    assert!(!cfg.peers[0].is_trusted); // Ainda não pareado/sem chave pública

    // Salva e recarrega
    cfg.save_to_file(&cfg_path).unwrap();
    let loaded = AppConfig::load_from_file(&cfg_path).unwrap();

    assert_eq!(loaded.peers.len(), 1);
    assert_eq!(loaded.peers[0].name, "Meu Linux");
    assert_eq!(loaded.peers[0].manual_port, Some(25800));
}

#[test]
fn test_dynamic_ip_update_preserves_crypto_identity() {
    let mut cfg = AppConfig::default();
    let trusted_pk = "7f3a0981884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08";

    cfg.peers.push(PeerConfig {
        name: "Notebook-Rafael".to_string(),
        ip_or_host: "192.168.1.20".to_string(),
        port: 25800,
        public_key_hex: trusted_pk.to_string(),
        is_trusted: true,
        alias: Some("Notebook-Rafael".to_string()),
        manual_ip: Some("192.168.1.20".to_string()),
        manual_port: Some(25800),
        last_seen_ip: None,
    });

    // O IP mudou via DHCP no dia seguinte para 192.168.1.35
    let updated = cfg.update_peer_ip_if_trusted(trusted_pk, "192.168.1.35");
    assert!(updated, "Deveria ter atualizado o IP do peer confiável");

    let peer = &cfg.peers[0];
    assert_eq!(peer.ip_or_host, "192.168.1.35");
    assert_eq!(peer.last_seen_ip.as_deref(), Some("192.168.1.20"));
    assert!(peer.is_trusted, "Confiança deve permanecer intacta após mudança de IP");
}

#[test]
fn test_untrusted_ip_spoofing_rejected() {
    let mut cfg = AppConfig::default();
    let legitimate_pk = "legitimate_key_11111111111111111111111111111111111111111111111111";

    cfg.peers.push(PeerConfig {
        name: "Servidor-Confiado".to_string(),
        ip_or_host: "192.168.1.50".to_string(),
        port: 25800,
        public_key_hex: legitimate_pk.to_string(),
        is_trusted: true,
        alias: None,
        manual_ip: None,
        manual_port: None,
        last_seen_ip: None,
    });

    // Uma máquina com o MESMO IP 192.168.1.50 tenta se conectar, mas apresenta outra chave
    let attacker_pk = "intruder_key_9999999999999999999999999999999999999999999999999999";

    assert!(cfg.is_peer_trusted(legitimate_pk));
    assert!(
        !cfg.is_peer_trusted(attacker_pk),
        "Máquina não deve ser confiada apenas pelo IP! A autoridade é a chave Ed25519."
    );
}

#[test]
fn test_remove_peer() {
    let mut cfg = AppConfig::default();
    cfg.add_or_update_manual_peer("192.168.1.80", 25800, Some("Temporario"), None);
    assert_eq!(cfg.peers.len(), 1);

    let removed = cfg.remove_peer("Temporario");
    assert!(removed);
    assert!(cfg.peers.is_empty());
}
