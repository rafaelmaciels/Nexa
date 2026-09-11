/**
 * Nexa — Simulador Interativo do Visual de Dispositivos (Soft Premium)
 * Alternância de layout (esquerda/direita) e simulação fluida de movimento do cursor.
 */

document.addEventListener('DOMContentLoaded', () => {
  const stage = document.getElementById('screensStage');
  const btnLeft = document.getElementById('btnPositionLeft');
  const btnRight = document.getElementById('btnPositionRight');
  const hostScreen = document.getElementById('screenHost');
  const remoteScreen = document.getElementById('screenRemote');
  const transitionBridge = document.getElementById('transitionBridge');
  const btnSimulateTransition = document.getElementById('btnSimulateTransition');

  if (!stage || !btnLeft || !btnRight || !hostScreen || !remoteScreen) return;

  let currentPosition = 'right'; // 'left' ou 'right'
  let isRemoteActive = false;

  // Função para alternar layout de monitores
  function updateMonitorLayout(position) {
    currentPosition = position;

    if (position === 'left') {
      btnLeft.classList.add('active');
      btnRight.classList.remove('active');
      stage.innerHTML = '';
      stage.appendChild(remoteScreen);
      stage.appendChild(transitionBridge);
      stage.appendChild(hostScreen);

      const bridgeCursor = transitionBridge.querySelector('.bridge-cursor-anim');
      if (bridgeCursor) {
        bridgeCursor.style.transform = 'scaleX(-1)';
      }
    } else {
      btnRight.classList.add('active');
      btnLeft.classList.remove('active');
      stage.innerHTML = '';
      stage.appendChild(hostScreen);
      stage.appendChild(transitionBridge);
      stage.appendChild(remoteScreen);

      const bridgeCursor = transitionBridge.querySelector('.bridge-cursor-anim');
      if (bridgeCursor) {
        bridgeCursor.style.transform = 'scaleX(1)';
      }
    }
  }

  btnLeft.addEventListener('click', () => updateMonitorLayout('left'));
  btnRight.addEventListener('click', () => updateMonitorLayout('right'));

  // Função para simular transição do cursor entre as telas
  function setCursorActiveState(remote) {
    isRemoteActive = remote;

    const hostStateText = hostScreen.querySelector('.device-state-text');
    const remoteStateText = remoteScreen.querySelector('.device-state-text');
    const hostPill = hostScreen.querySelector('.device-status-pill');
    const remotePill = remoteScreen.querySelector('.device-status-pill');

    if (remote) {
      // Cursor no Linux
      hostScreen.classList.remove('active-device');
      remoteScreen.classList.add('active-device');

      if (hostPill && hostStateText) {
        hostPill.style.background = 'var(--bg-subtle)';
        hostPill.style.color = 'var(--text-secondary)';
        hostStateText.textContent = 'Aguardando cursor';
      }
      if (remotePill && remoteStateText) {
        remotePill.style.background = 'var(--primary-light)';
        remotePill.style.color = 'var(--primary)';
        remoteStateText.textContent = 'Cursor ativo no Linux';
      }

      if (btnSimulateTransition) {
        btnSimulateTransition.innerHTML = '<span>Retornar cursor ao Windows</span>';
      }
    } else {
      // Cursor no Windows (Host)
      remoteScreen.classList.remove('active-device');
      hostScreen.classList.add('active-device');

      if (hostPill && hostStateText) {
        hostPill.style.background = 'var(--primary-light)';
        hostPill.style.color = 'var(--primary)';
        hostStateText.textContent = 'Mouse e teclado ativos aqui';
      }
      if (remotePill && remoteStateText) {
        remotePill.style.background = 'var(--bg-subtle)';
        remotePill.style.color = 'var(--text-secondary)';
        remoteStateText.textContent = 'Pronto para receber o cursor';
      }

      if (btnSimulateTransition) {
        btnSimulateTransition.innerHTML = '<span>Simular movimento entre telas</span>';
      }
    }
  }

  if (btnSimulateTransition) {
    btnSimulateTransition.addEventListener('click', () => {
      setCursorActiveState(!isRemoteActive);
    });
  }

  // Tecla de Emergência Esc ou Scroll Lock para devolver o foco
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' || e.key === 'ScrollLock') {
      if (isRemoteActive) {
        setCursorActiveState(false);
      }
    }
  });
});
