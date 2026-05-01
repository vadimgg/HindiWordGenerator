/**
 * Quick export controller.
 *
 * Responsible for: chapter/group-level one-click Anki exports from the Words
 * and Sentences pages. The Deliver tab remains the advanced custom workflow.
 *
 * Dependencies: anki/connect.js, anki/export.js, data.js.
 */
// Responsible for: quick Anki exports for whole word/sentence groups

import { checkAnkiConnect } from '../anki/connect.js';
import { sendToAnki, sendSentencesToAnki } from '../anki/export.js';
import { getAllWords, getAllSentences } from '../data.js';

const plural = n => (n === 1 ? '' : 's');

function deckPart(title) {
  return (title || 'Chapter').replace(/\s+/g, ' ').trim();
}

function defaultDeck(type, title) {
  return `Hindi::${deckPart(title)}::${type === 'words' ? 'Words' : 'Sentences'}`;
}

function setButtonState(button, label, state = '') {
  button.textContent = label;
  button.dataset.quickExportState = state;
}

function groupCards(button, type) {
  const wrapper = button.closest('.card-group-wrapper');
  const list = wrapper?.nextElementSibling;
  if (!list) return [];
  const selector = type === 'words' ? '[data-word-card]' : '[data-sentence-index]';
  const attr = type === 'words' ? 'wordCard' : 'sentenceIndex';
  const source = type === 'words' ? getAllWords() : getAllSentences();
  return [...list.querySelectorAll(selector)]
    .map(card => Number(card.dataset[attr]))
    .filter(index => Number.isInteger(index) && index >= 0)
    .map(index => source[index])
    .filter(Boolean);
}

function buildResultMessage(type, result) {
  const label = type === 'words' ? 'word' : 'sentence';
  if (result.added === 0 && result.skipped > 0) {
    return `All ${result.skipped} ${label}${plural(result.skipped)} already exist`;
  }
  if (result.skipped > 0) {
    return `${result.added} added · ${result.skipped} skipped`;
  }
  return `${result.added} ${label}${plural(result.added)} sent`;
}

async function handleQuickExport(button) {
  const type = button.dataset.quickExport;
  if (type !== 'words' && type !== 'sentences') return;
  const title = button.dataset.quickExportTitle ?? '';
  const deckName = defaultDeck(type, title);
  const cards = groupCards(button, type);
  if (cards.length === 0) {
    setButtonState(button, 'No cards', 'warning');
    return;
  }

  button.disabled = true;
  setButtonState(button, 'Checking...', '');

  try {
    if (!(await checkAnkiConnect())) {
      setButtonState(button, 'Open Anki', 'warning');
      return;
    }

    setButtonState(button, 'Sending...', '');
    const result = type === 'words'
      ? await sendToAnki(cards, deckName)
      : await sendSentencesToAnki(cards, deckName);
    setButtonState(button, buildResultMessage(type, result), 'success');
  } catch (error) {
    setButtonState(button, 'Export failed', 'warning');
    console.error('[quick-export]', error);
  } finally {
    button.disabled = false;
  }
}

export function initQuickExport() {
  document.addEventListener('click', event => {
    const button = event.target.closest('[data-quick-export]');
    if (!button) return;
    event.preventDefault();
    event.stopPropagation();
    handleQuickExport(button);
  });
}
