/**
 * Data health panel controller.
 *
 * Responsible for: keeping the Deliver page health summary in sync with the
 * current word and sentence selection.
 *
 * Dependencies: data.js, state/selection.js, audioHelpers.ts.
 */
// Responsible for: updating the Deliver data health panel from selected cards

import { getAllWords, getAllSentences } from '../data.js';
import { getSelectedWordIndices, getSelectedSentenceIndices } from '../state/selection.js';
import { resolveWordAudioSrc, resolveSentenceAudioSrc } from '../../utils/audioHelpers.ts';

const setText = (id, value) => {
  const el = document.getElementById(id);
  if (el) el.textContent = String(value);
};

function countMissingAudio(words, sentences) {
  const missingWords = words.filter(word => !resolveWordAudioSrc(word)).length;
  const missingSentences = sentences.filter(sentence => !resolveSentenceAudioSrc(sentence)).length;
  return missingWords + missingSentences;
}

function updateDataHealthPanel() {
  const allWords = getAllWords();
  const allSentences = getAllSentences();
  const selectedWords = getSelectedWordIndices().map(i => allWords[i]).filter(Boolean);
  const selectedSentences = getSelectedSentenceIndices().map(i => allSentences[i]).filter(Boolean);
  const selectedTotal = selectedWords.length + selectedSentences.length;
  const selectedMissingAudio = countMissingAudio(selectedWords, selectedSentences);

  setText('data-health-selected-total', selectedTotal);
  setText('data-health-selected-words', selectedWords.length);
  setText('data-health-selected-sentences', selectedSentences.length);
  setText('data-health-selected-missing-audio', selectedMissingAudio);
}

export function initDataHealth() {
  updateDataHealthPanel();
  window.addEventListener('selectionchange', updateDataHealthPanel);
}
