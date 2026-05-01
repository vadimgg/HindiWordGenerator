/**
 * QA page interactions.
 *
 * Responsible for: wiring QA issue actions, especially jumping from an issue
 * row to the source sentence card.
 *
 * Dependencies: state/tabs.js.
 */
// Responsible for: QA issue interactions

import { switchTab } from '../state/tabs.js';

function revealSentenceCard(index) {
  switchTab('sentences');
  document.getElementById('ps-mode-web')?.click();

  const card = document.getElementById(`sentence-card-${index}`);
  if (!card) return;
  card.classList.remove('is-collapsed');
  card.querySelector('.card-header')?.setAttribute('aria-expanded', 'true');
  card.scrollIntoView({ behavior: 'smooth', block: 'center' });
  card.classList.add('qa-jump-highlight');
  window.setTimeout(() => card.classList.remove('qa-jump-highlight'), 1800);
}

export function initQA() {
  document.addEventListener('click', event => {
    const button = event.target.closest('[data-qa-jump-sentence]');
    if (!button) return;
    const index = Number(button.dataset.qaJumpSentence);
    if (!Number.isInteger(index) || index < 0) return;
    revealSentenceCard(index);
  });
}
