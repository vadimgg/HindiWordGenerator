/**
 * Data health panel controller.
 *
 * Responsible for: keeping the Deliver page health summary in sync with the
 * current word and sentence selection.
 *
 * Dependencies: data.js, state/selection.js, quality/sentenceTokens.js, audioHelpers.ts.
 */
// Responsible for: updating the Deliver data health panel from selected cards

import { getAllWords, getAllSentences } from '../data.js';
import { getSelectedWordIndices, getSelectedSentenceIndices } from '../state/selection.js';
import { countSentenceTokenIssues, hasExactSentenceTokens } from '../quality/sentenceTokens.js';
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

function issueTitle(item) {
  if (item.type === 'word') return item.card.hindi || item.card.english || 'Word card';
  return item.card.hindi || item.card.english || 'Sentence card';
}

function issueSubtitle(item) {
  const prefix = item.type === 'word' ? 'Word' : 'Sentence';
  return `${prefix} · ${item.reason}`;
}

function collectIssues(words, sentences) {
  const wordIssues = words
    .filter(word => !resolveWordAudioSrc(word))
    .map(card => ({ type: 'word', reason: 'missing audio', card }));
  const sentenceAudioIssues = sentences
    .filter(sentence => !resolveSentenceAudioSrc(sentence))
    .map(card => ({ type: 'sentence', reason: 'missing audio', card }));
  const sentenceTokenIssues = sentences
    .filter(sentence => !hasExactSentenceTokens(sentence))
    .map(card => ({ type: 'sentence', reason: 'tokens do not reconstruct exactly', card }));

  return [...wordIssues, ...sentenceAudioIssues, ...sentenceTokenIssues];
}

function updateReviewDetails(issues) {
  const count = document.getElementById('data-health-review-count');
  const empty = document.getElementById('data-health-review-empty');
  const list = document.getElementById('data-health-issue-list');
  if (!count || !empty || !list) return;

  count.textContent = `${issues.length} ${issues.length === 1 ? 'issue' : 'issues'}`;
  empty.hidden = issues.length > 0;
  list.innerHTML = '';
  list.hidden = issues.length === 0;

  for (const issue of issues.slice(0, 8)) {
    const item = document.createElement('li');
    const title = document.createElement('span');
    const subtitle = document.createElement('small');
    title.textContent = issueTitle(issue);
    subtitle.textContent = issueSubtitle(issue);
    item.append(title, subtitle);
    list.append(item);
  }

  if (issues.length > 8) {
    const item = document.createElement('li');
    const title = document.createElement('span');
    title.textContent = `${issues.length - 8} more selected issues`;
    item.append(title);
    list.append(item);
  }
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
  const selectedTokenIssues = countSentenceTokenIssues(selectedSentences);
  const selectedIssues = collectIssues(selectedWords, selectedSentences);

  setText('data-health-selected-total', selectedTotal);
  setText('data-health-selected-words', selectedWords.length);
  setText('data-health-selected-sentences', selectedSentences.length);
  setText('data-health-selected-missing-audio', selectedMissingAudio);
  setText('data-health-selected-token-issues', selectedTokenIssues);
  updateStatus(selectedMissingAudio, selectedTokenIssues);
  updateReviewDetails(selectedIssues);
}

export function initDataHealth() {
  updateDataHealthPanel();
  window.addEventListener('selectionchange', updateDataHealthPanel);
}
