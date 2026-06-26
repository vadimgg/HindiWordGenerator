function setViewMode(page, mode) {
  const pageEl = document.getElementById(`page-${page}`);
  if (!pageEl) return;
  pageEl.classList.toggle('mode-web', mode === 'web');
  pageEl.classList.toggle('mode-anki', mode === 'anki');

  const prefix = page === 'words' ? 'pw' : 'ps';
  const webBtn = document.getElementById(`${prefix}-mode-web`);
  const ankiBtn = document.getElementById(`${prefix}-mode-anki`);
  if (webBtn) {
    webBtn.classList.toggle('is-active', mode === 'web');
    webBtn.setAttribute('aria-pressed', String(mode === 'web'));
  }
  if (ankiBtn) {
    ankiBtn.classList.toggle('is-active', mode === 'anki');
    ankiBtn.setAttribute('aria-pressed', String(mode === 'anki'));
  }
}

export function wireViewMode(page, prefix) {
  document.getElementById(`${prefix}-mode-web`)?.addEventListener('click', () => setViewMode(page, 'web'));
  document.getElementById(`${prefix}-mode-anki`)?.addEventListener('click', () => setViewMode(page, 'anki'));
}
