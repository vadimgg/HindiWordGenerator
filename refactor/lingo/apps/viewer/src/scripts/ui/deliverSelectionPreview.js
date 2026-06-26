/**
 * Deliver selection preview — renders selected cards and count summaries.
 */

import { getAllSentences } from '../data.js';
import { getSelectedSentenceIndices, getSelectedWordObjects } from '../state/selection.js';
import { resolveSentenceAudioSrc, resolveWordAudioSrc } from '../../utils/audioHelpers.ts';
import { syncAllDeckPreviews } from './deckControls.js';
import { renderSentenceRow, renderWordRow } from './deliverRows.js';

const plural = n => (n !== 1 ? 's' : '');

export function getSelectedDeliverItems() {
  const words = getSelectedWordObjects();
  const allSentences = getAllSentences();
  const sentences = getSelectedSentenceIndices().map(index => allSentences[index]).filter(Boolean);
  return { words, sentences };
}

export function populateDeliverPreview() {
  const { words, sentences } = getSelectedDeliverItems();

  const wordRows = document.getElementById('export-word-rows');
  const sentRows = document.getElementById('export-sent-rows');
  const wordCountEl = document.getElementById('export-word-count');
  const sentCountEl = document.getElementById('export-sent-count');
  const metaEl = document.getElementById('deliver-action-meta');
  const confirmWordCount = document.getElementById('deliver-confirm-word-count');
  const confirmSentLine = document.getElementById('deliver-confirm-sent-line');
  const confirmSentCount = document.getElementById('deliver-confirm-sent-count');
  const deliverMain = document.getElementById('deliver-main');
  const deliverEmpty = document.getElementById('deliver-empty');

  if (wordCountEl) wordCountEl.textContent = String(words.length);
  if (sentCountEl) sentCountEl.textContent = String(sentences.length);
  if (confirmWordCount) confirmWordCount.textContent = `${words.length} word card${plural(words.length)}`;
  if (confirmSentCount) confirmSentCount.textContent = `${sentences.length} sentence card${plural(sentences.length)}`;
  if (confirmSentLine) confirmSentLine.style.display = sentences.length > 0 ? '' : 'none';

  const total = words.length + sentences.length;
  if (metaEl) {
    if (total === 0) {
      metaEl.textContent = 'Select words or sentences to begin';
    } else if (words.length > 0 && sentences.length > 0) {
      const strong = document.createElement('strong');
      strong.textContent = `${total} cards`;
      metaEl.replaceChildren(strong, ' across 2 decks');
    } else {
      metaEl.textContent = `${total} card${plural(total)} ready to export`;
    }
  }

  if (deliverMain && deliverEmpty) {
    const isEmpty = total === 0;
    deliverMain.style.display = isEmpty ? 'none' : '';
    deliverEmpty.style.display = isEmpty ? '' : 'none';
  }

  syncAllDeckPreviews();

  if (wordRows) {
    wordRows.replaceChildren(...words.map(word =>
      renderWordRow(word, Boolean(resolveWordAudioSrc(word)))
    ));
  }
  if (sentRows) {
    sentRows.replaceChildren(...sentences.map(sentence =>
      renderSentenceRow(sentence, Boolean(resolveSentenceAudioSrc(sentence)))
    ));
  }
}
