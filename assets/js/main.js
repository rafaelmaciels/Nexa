/**
 * Nexa Website — Script Principal
 * Funcionalidades: Menu mobile, cópia para clipboard, abas de plataforma e FAQ.
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

    // Fecha menu ao clicar em links
    navMenu.querySelectorAll('.nav-link').forEach(link => {
      link.addEventListener('click', () => navMenu.classList.remove('open'));
    });
  }

  // 2. Sistema Global de Cópia para Clipboard
  document.querySelectorAll('.btn-copy-code, .btn-copy').forEach(btn => {
    btn.addEventListener('click', async () => {
      const codeTarget = btn.getAttribute('data-clipboard-target');
      let textToCopy = '';

      if (codeTarget) {
        const targetElement = document.querySelector(codeTarget);
        if (targetElement) {
          textToCopy = targetElement.innerText || targetElement.textContent;
        }
      } else if (btn.getAttribute('data-clipboard-text')) {
        textToCopy = btn.getAttribute('data-clipboard-text');
      } else {
        const parentCodeBox = btn.closest('.code-box') || btn.closest('.code-block-container');
        if (parentCodeBox) {
          const codeEl = parentCodeBox.querySelector('.code-snippet') || parentCodeBox.querySelector('code') || parentCodeBox.querySelector('pre');
          if (codeEl) textToCopy = codeEl.innerText;
        }
      }

      if (!textToCopy) return;

      try {
        if (navigator.clipboard && window.isSecureContext) {
          await navigator.clipboard.writeText(textToCopy.trim());
        } else {
          // Fallback para contextos não-HTTPS
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
        btn.classList.add('copied');
        btn.innerHTML = `
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="20 6 9 17 4 12"></polyline>
          </svg>
          <span>Copiado!</span>
        `;

        setTimeout(() => {
          btn.classList.remove('copied');
          btn.innerHTML = originalHtml;
        }, 2200);
      } catch (err) {
        console.error('Falha ao copiar:', err);
      }
    });
  });

  // 3. Alternador de Abas de Plataforma (Windows / Linux)
  const tabButtons = document.querySelectorAll('.platform-tab-btn');
  const tabPanels = document.querySelectorAll('.platform-tab-panel');

  if (tabButtons.length > 0 && tabPanels.length > 0) {
    tabButtons.forEach(button => {
      button.addEventListener('click', () => {
        const targetTab = button.getAttribute('data-platform');

        tabButtons.forEach(b => b.classList.remove('active'));
        tabPanels.forEach(p => p.classList.remove('active'));

        button.classList.add('active');
        const targetPanel = document.getElementById(`tab-content-${targetTab}`);
        if (targetPanel) {
          targetPanel.classList.add('active');
        }
      });
    });
  }

  // 4. FAQ Accordion
  const faqQuestions = document.querySelectorAll('.faq-question');
  faqQuestions.forEach(question => {
    question.addEventListener('click', () => {
      const item = question.parentElement;
      const isOpen = item.classList.contains('open');

      // Fecha todos os outros itens para manter o accordion limpo
      document.querySelectorAll('.faq-item').forEach(i => i.classList.remove('open'));

      if (!isOpen) {
        item.classList.add('open');
      }
    });
  });

  // 5. ScrollSpy para Sidebar de Documentação
  const docsLinks = document.querySelectorAll('.docs-nav-link');
  if (docsLinks.length > 0) {
    const sections = Array.from(docsLinks)
      .map(link => {
        const id = link.getAttribute('href');
        return id && id.startsWith('#') ? document.querySelector(id) : null;
      })
      .filter(Boolean);

    window.addEventListener('scroll', () => {
      let currentSection = null;
      const scrollPos = window.scrollY + 140;

      for (const section of sections) {
        if (section.offsetTop <= scrollPos) {
          currentSection = section;
        }
      }

      if (currentSection) {
        docsLinks.forEach(link => {
          link.classList.toggle('active', link.getAttribute('href') === `#${currentSection.id}`);
        });
      }
    }, { passive: true });
  }
});
