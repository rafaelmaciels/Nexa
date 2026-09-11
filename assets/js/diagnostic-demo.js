/**
 * Nexa — Simulador de Diagnóstico de Rede e Criptografia
 * Verificação interativa das 5 etapas essenciais de conectividade local.
 */

document.addEventListener('DOMContentLoaded', () => {
  const btnRunDiag = document.getElementById('btnRunDiagnostic');
  const diagItems = document.querySelectorAll('.tech-step-row.diagnostic-item, .diagnostic-item');

  if (!btnRunDiag || diagItems.length === 0) return;

  const stepsData = [
    { label: '✓ OK', text: 'Conectividade IP' },
    { label: '✓ Aberta', text: 'Porta TCP 25800' },
    { label: '✓ Respondendo', text: 'Daemon Nexa' },
    { label: '✓ Cifrado', text: 'Criptografia Curve25519' },
    { label: '✓ Pronto', text: 'Driver Linux /dev/uinput' }
  ];

  let isRunning = false;

  btnRunDiag.addEventListener('click', async () => {
    if (isRunning) return;
    isRunning = true;

    btnRunDiag.disabled = true;
    btnRunDiag.innerHTML = `
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" style="animation: spin 1s linear infinite; margin-right: 6px;">
        <circle cx="12" cy="12" r="10" stroke-opacity="0.25"></circle>
        <path d="M12 2a10 10 0 0 1 10 10"></path>
      </svg>
      <span>Verificando...</span>
    `;

    // Reseta status para testando
    diagItems.forEach(item => {
      const statusEl = item.querySelector('.diagnostic-status, .tech-step-badge');
      if (statusEl) {
        statusEl.textContent = '○ Verificando...';
        statusEl.style.color = 'var(--text-muted)';
      }
    });

    // Simula validação sequencial com micro-delays suaves
    for (let i = 0; i < diagItems.length; i++) {
      await new Promise(resolve => setTimeout(resolve, 220));
      const item = diagItems[i];
      const statusEl = item.querySelector('.diagnostic-status, .tech-step-badge');
      const step = stepsData[i];

      if (statusEl && step) {
        statusEl.textContent = step.label;
        statusEl.style.color = 'var(--primary)';
      }
    }

    btnRunDiag.disabled = false;
    btnRunDiag.innerHTML = '<span>Executar verificação completa</span>';
    isRunning = false;
  });
});

// Animação de rotação para o spinner
const style = document.createElement('style');
style.textContent = `
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
`;
document.head.appendChild(style);
