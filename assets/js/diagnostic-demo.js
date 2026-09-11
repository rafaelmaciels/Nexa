/**
 * Nexa — Simulador Interativo de Diagnóstico em Tempo Real
 * Demonstração do funcionamento da verificação em 5 etapas da CLI e da GUI.
 */

document.addEventListener('DOMContentLoaded', () => {
  const btnRunDiag = document.getElementById('btnRunDiagnostic');
  const diagItems = document.querySelectorAll('.diagnostic-item');
  const btnToggleService = document.getElementById('btnToggleService');

  if (!btnRunDiag || diagItems.length === 0) return;

  const stepsData = [
    { label: '✓ OK', class: 'status-check-ok', text: 'Conectividade IP (Rede Local)' },
    { label: '✓ Aberta', class: 'status-check-ok', text: 'Porta TCP 25800 (Firewall)' },
    { label: '✓ Respondendo', class: 'status-check-ok', text: 'Serviço Nexa (Daemon Ativo)' },
    { label: '✓ Seguro', class: 'status-check-ok', text: 'Handshake ChaCha20-Poly1305' },
    { label: '✓ Ativo', class: 'status-check-ok', text: 'Permissão Linux /dev/uinput' }
  ];

  let isRunning = false;

  btnRunDiag.addEventListener('click', async () => {
    if (isRunning) return;
    isRunning = true;

    btnRunDiag.disabled = true;
    btnRunDiag.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="animation: spin 1s linear infinite;">
        <circle cx="12" cy="12" r="10" stroke-opacity="0.25"></circle>
        <path d="M12 2a10 10 0 0 1 10 10"></path>
      </svg>
      <span>Verificando Rede...</span>
    `;

    // Reseta todos os status para pendente
    diagItems.forEach(item => {
      const statusEl = item.querySelector('.diagnostic-status');
      if (statusEl) {
        statusEl.className = 'diagnostic-status';
        statusEl.innerHTML = '<span style="color: var(--text-dim);">○ Testando...</span>';
      }
    });

    // Animação passo a passo (simulando 200ms por etapa)
    for (let i = 0; i < diagItems.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 280));
      const item = diagItems[i];
      const statusEl = item.querySelector('.diagnostic-status');
      const step = stepsData[i];

      if (statusEl && step) {
        statusEl.className = `diagnostic-status ${step.class}`;
        statusEl.textContent = step.label;
      }
    }

    btnRunDiag.disabled = false;
    btnRunDiag.innerHTML = `
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <circle cx="11" cy="11" r="8"></circle>
        <line x1="21" y1="21" x2="16.65" y2="16.65"></line>
      </svg>
      <span>Testar Conexão</span>
    `;
    isRunning = false;
  });

  // Botão de parar/iniciar serviço
  if (btnToggleService) {
    let serviceActive = true;
    btnToggleService.addEventListener('click', () => {
      serviceActive = !serviceActive;
      if (serviceActive) {
        btnToggleService.className = 'btn btn-primary';
        btnToggleService.innerHTML = '<span>⏹ Parar Serviço</span>';
      } else {
        btnToggleService.className = 'btn btn-secondary';
        btnToggleService.innerHTML = '<span>▶ Iniciar Serviço</span>';
      }
    });
  }
});

// Estilo de rotação para spinner
const style = document.createElement('style');
style.textContent = `
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
`;
document.head.appendChild(style);
