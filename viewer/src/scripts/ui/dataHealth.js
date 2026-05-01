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

function hasExactSentenceTokens(sentence) {
  if (!Array.isArray(sentence.tokens) || sentence.tokens.length === 0) return false;
  const joinedHindi = sentence.tokens.map(token => token.hindi ?? '').join('');
  const joinedRoman = sentence.tokens.map(token => token.roman ?? '').join('');
  return joinedHindi === (sentence.hindi ?? '') && joinedRoman === (sentence.romanisation ?? '');
}

function countMissingAudio(words, sentences) {
  const missingWords = words.filter(word => !resolveWordAudioSrc(word)).length;
  const missingSentences = sentences.filter(sentence => !resolveSentenceAudioSrc(sentence)).length;
  return missingWords + missingSentences;
}

function updateStatus(selectedMissingAudio, selectedTokenIssues) {
  const row = document.getElementById('data-health-selected-status');
  const label = document.getElementById('data-health-selected-status-label');
  if (!row || !label) return;

  const hasWarnings = selectedMissingAudio > 0 || selectedTokenIssues > 0;
  row.classList.toggle('has-warning', hasWarnings);
  label.textContent = hasWarnings ? 'Review' : 'Ready';
}

function updateDataHealthPanel() {
  const allWords = getAllWords();
  const allSentences = getAllSentences();
  const selectedWords = getSelectedWordIndices().map(i => allWords[i]).filter(Boolean);
  const selectedSentences = getSelectedSentenceIndices().map(i => allSentences[i]).filter(Boolean);
  const selectedTotal = selectedWords.length + selectedSentences.length;
  const selectedMissingAudio = countMissingAudio(selectedWords, selectedSentences);
  const selectedTokenIssues = selectedSentences.filter(sentence => !hasExactSentenceTokens(sentence)).length;

  setText('data-health-selected-total', selectedTotal);
  setText('data-health-selected-words', selectedWords.length);
  setText('data-health-selected-sentences', selectedSentences.length);
  setText('data-health-selected-missing-audio', selectedMissingAudio);
  setText('data-health-selected-token-issues', selectedTokenIssues);
  updateStatus(selectedMissingAudio, selectedTokenIssues);
}

export function initDataHealth() {
  updateDataHealthPanel();
  window.addEventListener('selectionchange', updateDataHealthPanel);
}
