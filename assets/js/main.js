/**
 * Nexa Website — Script Principal (Soft Premium / Calm Tech)
 * Funcionalidades: Conexão interativa, modo host/guest, drawers expansíveis,
 * cópia para clipboard, menu mobile e acessibilidade.
 */

document.addEventListener('DOMContentLoaded', () => {
  // 1. Menu Mobile Toggle
  const mobileToggle = document.querySelector('.btn-mobile-toggle');
  const navMenu = document.querySelector('.nav-menu');

  if (mobileToggle && navMenu) {
    mobileToggle.addEventListener('click', () => {
      navMenu.classList.toggle('open');
      const isOpen = navMenu.classList.contains('open');
      mobileToggle.setAttribute('aria-expanded', isOpen);
    });

    navMenu.querySelectorAll('.nav-link').forEach(link => {
      link.addEventListener('click', () => navMenu.classList.remove('open'));
    });
  }

  // 2. Botão Principal "Conectar ao Nexa" com Feedback Visual Suave
  const btnConnect = document.getElementById('btnConnectMain');
  const headerStatusPill = document.getElementById('headerStatusPill');
  const headerStatusText = document.getElementById('headerStatusText');
  const serviceBadge = document.getElementById('serviceStatusBadge');

  if (btnConnect) {
    let isConnected = false;

    btnConnect.addEventListener('click', () => {
      isConnected = !isConnected;

      if (isConnected) {
        btnConnect.classList.add('connected');
        btnConnect.innerHTML = `
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          <span>Conectado</span>
        `;

        if (headerStatusPill && headerStatusText) {
          headerStatusText.textContent = 'Conectado';
          headerStatusPill.style.background = 'var(--primary-light)';
          headerStatusPill.style.borderColor = 'var(--primary-light-border)';
        }

        if (serviceBadge) {
          serviceBadge.textContent = 'Ativo';
          serviceBadge.style.color = 'var(--primary)';
        }
      } else {
        btnConnect.classList.remove('connected');
        btnConnect.innerHTML = '<span>Conectar ao Nexa</span>';

        if (headerStatusPill && headerStatusText) {
          headerStatusText.textContent = 'Pronto para conectar';
          headerStatusPill.style.background = 'var(--bg-surface)';
          headerStatusPill.style.borderColor = 'var(--border-light)';
        }
      }
    });
  }

  // 3. Alternador de Modo (Este computador / Outro computador)
  const btnModeServer = document.getElementById('btnModeServer');
  const btnModeClient = document.getElementById('btnModeClient');
  const screenHost = document.getElementById('screenHost');
  const screenRemote = document.getElementById('screenRemote');

  if (btnModeServer && btnModeClient) {
    btnModeServer.addEventListener('click', () => {
      btnModeServer.classList.add('active');
      btnModeClient.classList.remove('active');
      if (screenHost) {
        const tag = screenHost.querySelector('.device-os-tag');
        if (tag) tag.textContent = 'Este computador';
      }
      if (screenRemote) {
        const tag = screenRemote.querySelector('.device-os-tag');
        if (tag) tag.textContent = 'Outro computador';
      }
    });

    btnModeClient.addEventListener('click', () => {
      btnModeClient.classList.add('active');
      btnModeServer.classList.remove('active');
      if (screenHost) {
        const tag = screenHost.querySelector('.device-os-tag');
        if (tag) tag.textContent = 'Outro computador';
      }
      if (screenRemote) {
        const tag = screenRemote.querySelector('.device-os-tag');
        if (tag) tag.textContent = 'Este computador';
      }
    });
  }

  // 4. Drawer de Detalhes Técnicos (Sob Demanda)
  const btnToggleTech = document.getElementById('btnToggleTechDetails');
  const techDrawer = document.getElementById('techDetailsDrawer');

  if (btnToggleTech && techDrawer) {
    btnToggleTech.addEventListener('click', () => {
      const isOpen = techDrawer.classList.toggle('open');
      btnToggleTech.setAttribute('aria-expanded', isOpen);
      const chevron = btnToggleTech.querySelector('svg');
      if (chevron) {
        chevron.style.transform = isOpen ? 'rotate(180deg)' : 'rotate(0deg)';
      }
      btnToggleTech.querySelector('span').textContent = isOpen 
        ? 'Ocultar detalhes técnicos' 
        : 'Ver detalhes técnicos';
    });
  }

  // 5. Drawer de Configuração Linux (Sob Demanda)
  const btnShowLinux = document.getElementById('btnShowLinuxConfig');
  const linuxDrawer = document.getElementById('linuxConfigDrawer');

  if (btnShowLinux && linuxDrawer) {
    btnShowLinux.addEventListener('click', () => {
      const isOpen = linuxDrawer.classList.toggle('open');
      btnShowLinux.setAttribute('aria-expanded', isOpen);
      const chevron = btnShowLinux.querySelector('svg');
      if (chevron) {
        chevron.style.transform = isOpen ? 'rotate(180deg)' : 'rotate(0deg)';
      }
      btnShowLinux.querySelector('span').textContent = isOpen 
        ? 'Ocultar configuração' 
        : 'Mostrar configuração';
    });
  }

  // 6. Sistema Global de Cópia para Clipboard
  document.querySelectorAll('.btn-copy-code, .btn-copy').forEach(btn => {
    btn.addEventListener('click', async () => {
      let textToCopy = btn.getAttribute('data-clipboard-text') || '';
      
      if (!textToCopy) {
        const parent = btn.closest('.terminal-box') || btn.closest('.input-ip-group');
        if (parent) {
          const codeEl = parent.querySelector('.terminal-code') || parent.querySelector('input');
          if (codeEl) textToCopy = codeEl.value || codeEl.innerText || '';
        }
      }

      if (!textToCopy) return;

      try {
        if (navigator.clipboard && window.isSecureContext) {
          await navigator.clipboard.writeText(textToCopy.trim());
        } else {
          const textArea = document.createElement('textarea');
          textArea.value = textToCopy.trim();
          textArea.style.position = 'fixed';
          textArea.style.opacity = '0';
          document.body.appendChild(textArea);
          textArea.focus();
          textArea.select();
          document.execCommand('copy');
          document.body.removeChild(textArea);
        }

        const originalHtml = btn.innerHTML;
        btn.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          <span>✓ Copiado</span>
        `;

        setTimeout(() => {
          btn.innerHTML = originalHtml;
        }, 2200);
      } catch (err) {
        console.error('Falha ao copiar:', err);
      }
    });
  });

  // 7. Suporte a FAQ Accordion (em subpáginas)
  const faqQuestions = document.querySelectorAll('.faq-question');
  faqQuestions.forEach(question => {
    question.addEventListener('click', () => {
      const item = question.parentElement;
      const isOpen = item.classList.contains('open');
      document.querySelectorAll('.faq-item').forEach(i => i.classList.remove('open'));
      if (!isOpen) {
        item.classList.add('open');
      }
    });
  });
});
