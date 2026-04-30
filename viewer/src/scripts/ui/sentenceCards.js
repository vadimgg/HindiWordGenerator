/**
 * Sentence card interaction — collapse/expand handling.
 *
 * Responsible for: toggling the is-collapsed state when the user clicks a
 * sentence card header inside #sentences-container.
 *
 * Dependencies: none.
 */
// Responsible for: sentence card collapse/expand toggling

/**
 * Initialises sentence card interactions:
 *   - Delegated click handler on #sentences-container for header collapse/expand.
 *
 * Must be called after DOMContentLoaded.
 *
 * @returns {void}
 */
export function initSentenceCards() {
  const sentencesContainer = document.getElementById('sentences-container');
  if (!sentencesContainer) return;

  sentencesContainer.addEventListener('click', e => {
    const header = e.target.closest('.card-header');
    if (!header) return;
    if (e.target.closest('.wc-audio-btn')) return;
    header.closest('article')?.classList.toggle('is-collapsed');
  });
}
