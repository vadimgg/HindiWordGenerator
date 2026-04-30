/**
 * Word card interaction — collapse/expand and deselect handling.
 *
 * Responsible for: toggling the is-collapsed state when the user clicks a card
 * header, handling the ✕ deselect button on each card, and using an
 * IntersectionObserver to highlight the corresponding sidebar row as the user
 * scrolls through the card list.
 *
 * Filter re-application after deselect is handled by the 'selectionchange'
 * CustomEvent that selection.js dispatches — no direct call to search.js needed.
 *
 * Dependencies: state/selection.js.
 */
// Responsible for: word card collapse/expand toggling and deselect button handling

import { setWordSelected, syncGroupCheckbox } from '../state/selection.js';

/**
 * Sets up an IntersectionObserver that highlights the matching sidebar row whenever
 * a word card scrolls into view.
 *
 * Assigns a sequential data-card-index attribute to each card article so the
 * observer can look up the sidebar row by index.
 *
 * @returns {void}
 */
function initScrollHighlight() {
  const wordCards = [...document.querySelectorAll('[data-word-card]')];
  const observer  = new IntersectionObserver(entries => {
    entries.forEach(entry => {
      if (!entry.isIntersecting) return;
      const cardIndex = entry.target.dataset.cardIndex;
      document.querySelectorAll('#idx-words .idx-row').forEach(r => r.classList.remove('is-active'));
      const activeRow = document.querySelector(`#idx-words .idx-row[data-index="${cardIndex}"]`);
      activeRow?.classList.add('is-active');
      activeRow?.scrollIntoView({ block: 'nearest' });
    });
  }, { threshold: 0.4 });

  wordCards.forEach((wrapper, i) => {
    const article = wrapper.querySelector('article');
    if (article) article.dataset.cardIndex = String(i);
    observer.observe(wrapper);
  });
}

/**
 * Initialises word card interactions:
 *   - Delegated click handler on #cards-container for header collapse and deselect.
 *   - IntersectionObserver that highlights the active sidebar row as cards scroll into view.
 *
 * Must be called after DOMContentLoaded.
 *
 * @returns {void}
 */
export function initCards() {
  const wordsContainer = document.getElementById('cards-container');
  if (wordsContainer) {
    wordsContainer.addEventListener('click', e => {
      const header = e.target.closest('.card-header');
      if (header && !e.target.closest('.deselect-btn, .wc-audio-btn')) {
        header.closest('article')?.classList.toggle('is-collapsed');
        return;
      }

      const deselectBtn = e.target.closest('.deselect-btn');
      if (!deselectBtn) return;
      const wordIndex = parseInt(deselectBtn.dataset.deselectIndex);
      const groupDate = document.querySelector(`#idx-words .idx-row[data-index="${wordIndex}"]`)
        ?.closest('.date-group')?.dataset.group ?? '';
      setWordSelected(wordIndex, false);
      syncGroupCheckbox(groupDate);
    });
  }

  initScrollHighlight();
}
