/**
 * Quick export controller.
 *
 * Responsible for: source/group-level one-click Anki exports from the Words
 * and Sentences pages. The Deliver tab remains the advanced custom workflow.
 *
 * Dependencies: anki/connect.js, anki/export.js, data.js.
 */
// Responsible for: quick Anki exports for whole word/sentence groups

import { checkAnkiConnect } from '../anki/connect.js';
import { quickExportDeckName } from '../anki/deckNames.js';
import { sendSentencesToAnki, sendToAnki } from '../anki/exportService.js';
import { getAllWords, getAllSentences } from '../data.js';

const plural = n => (n === 1 ? '' : 's');

function setButtonState(button, label, state = '', detail = '') {
  button.textContent = label;
  button.dataset.quickExportState = state;
  if (detail) button.title = detail;
  button.setAttribute('aria-label', detail || label);
}

function describeDeckAction(type, deckName, count) {
  const label = type === 'words' ? 'word' : 'sentence';
  return `Export ${count} ${label}${plural(count)} to ${deckName}`;
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
  const deckName = quickExportDeckName(type, title);
  const cards = groupCards(button, type);
  const actionLabel = describeDeckAction(type, deckName, cards.length);
  if (cards.length === 0) {
    setButtonState(button, 'No cards', 'warning', `No cards found for ${deckName}`);
    return;
  }

  button.disabled = true;
  setButtonState(button, 'Checking...', '', actionLabel);

  try {
    if (!(await checkAnkiConnect())) {
      setButtonState(button, 'Open Anki', 'warning', `${actionLabel}. AnkiConnect is offline.`);
      return;
    }

    setButtonState(button, 'Sending...', '', actionLabel);
    const result = type === 'words'
      ? await sendToAnki(cards, deckName)
      : await sendSentencesToAnki(cards, deckName);
    setButtonState(button, buildResultMessage(type, result), 'success', `${actionLabel}. Target deck: ${deckName}`);
  } catch (error) {
    setButtonState(button, 'Export failed', 'warning', `${actionLabel}. ${error.message}`);
    console.error('[quick-export]', error);
  } finally {
    button.disabled = false;
  }
}

export function initQuickExport() {
  document.querySelectorAll('[data-quick-export]').forEach(button => {
    const type = button.dataset.quickExport;
    const title = button.dataset.quickExportTitle ?? '';
    const deckName = quickExportDeckName(type, title);
    setButtonState(button, button.textContent.trim() || 'Quick export', '', `Target deck: ${deckName}`);
  });

  document.addEventListener('click', event => {
    const button = event.target.closest('[data-quick-export]');
    if (!button) return;
    event.preventDefault();
    event.stopPropagation();
    handleQuickExport(button);
  });
}
