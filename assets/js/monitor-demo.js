/**
 * Nexa — Simulador Interativo do Canvas de Monitores
 * Réplica interativa do layout espacial de telas visto no painel GUI oficial.
 */

document.addEventListener('DOMContentLoaded', () => {
  const stage = document.getElementById('screensStage');
  const btnLeft = document.getElementById('btnPositionLeft');
  const btnRight = document.getElementById('btnPositionRight');
  const hostScreen = document.getElementById('screenHost');
  const remoteScreen = document.getElementById('screenRemote');
  const transitionBridge = document.getElementById('transitionBridge');
  const cursorStatusBadge = document.getElementById('cursorStatusBadge');
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

      // Inverte a direção visual da seta
      const arrowSvg = transitionBridge.querySelector('svg');
      if (arrowSvg) {
        arrowSvg.style.transform = 'rotate(180deg)';
      }
    } else {
      btnRight.classList.add('active');
      btnLeft.classList.remove('active');
      stage.innerHTML = '';
      stage.appendChild(hostScreen);
      stage.appendChild(transitionBridge);
      stage.appendChild(remoteScreen);

      const arrowSvg = transitionBridge.querySelector('svg');
      if (arrowSvg) {
        arrowSvg.style.transform = 'rotate(0deg)';
      }
    }
  }

  // Eventos de clique nas posições
  btnLeft.addEventListener('click', () => updateMonitorLayout('left'));
  btnRight.addEventListener('click', () => updateMonitorLayout('right'));

  // Função para simular transição do cursor
  function setCursorActiveState(remote) {
    isRemoteActive = remote;

    const hostStatus = hostScreen.querySelector('.screen-status-indicator');
    const remoteStatus = remoteScreen.querySelector('.screen-status-indicator');

    if (remote) {
      // Cursor no Linux
      hostScreen.classList.remove('active-screen');
      remoteScreen.classList.add('active-screen');

      if (hostStatus) {
        hostStatus.className = 'screen-status-indicator status-passive';
        hostStatus.innerHTML = '<span>○ Cursor Remoto</span>';
      }
      if (remoteStatus) {
        remoteStatus.className = 'screen-status-indicator status-active';
        remoteStatus.innerHTML = '<span class="status-dot"></span><span>● Controlando Linux</span>';
      }

      if (cursorStatusBadge) {
        cursorStatusBadge.style.color = '#c084fc';
        cursorStatusBadge.style.borderColor = 'rgba(192, 132, 252, 0.4)';
        cursorStatusBadge.style.background = 'rgba(192, 132, 252, 0.12)';
        cursorStatusBadge.innerHTML = '<span class="pulse-dot" style="background:#c084fc;box-shadow:0 0 10px #c084fc;"></span> Cursor Ativo: Linux Laptop (Remoto)';
      }

      if (btnSimulateTransition) {
        btnSimulateTransition.innerHTML = '<span>🔙 Pressione Esc para Retornar</span>';
      }
    } else {
      // Cursor no Windows (Host)
      remoteScreen.classList.remove('active-screen');
      hostScreen.classList.add('active-screen');

      if (hostStatus) {
        hostStatus.className = 'screen-status-indicator status-active';
        hostStatus.innerHTML = '<span class="status-dot"></span><span>● Cursor Local Ativo</span>';
      }
      if (remoteStatus) {
        remoteStatus.className = 'screen-status-indicator status-passive';
        remoteStatus.innerHTML = '<span>○ Controlado pela rede local</span>';
      }

      if (cursorStatusBadge) {
        cursorStatusBadge.style.color = '#34d399';
        cursorStatusBadge.style.borderColor = 'rgba(16, 185, 129, 0.3)';
        cursorStatusBadge.style.background = 'rgba(16, 185, 129, 0.12)';
        cursorStatusBadge.innerHTML = '<span class="pulse-dot"></span> Cursor Ativo: Windows (Host)';
      }

      if (btnSimulateTransition) {
        btnSimulateTransition.innerHTML = '<span>✨ Simular Transição de Borda</span>';
      }
    }
  }

  if (btnSimulateTransition) {
    btnSimulateTransition.addEventListener('click', () => {
      setCursorActiveState(!isRemoteActive);
    });
  }

  // Tecla de Emergência Esc ou Scroll Lock
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' || e.key === 'ScrollLock') {
      if (isRemoteActive) {
        setCursorActiveState(false);
      }
    }
  });
});
