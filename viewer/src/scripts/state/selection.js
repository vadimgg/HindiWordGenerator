/**
 * Word and sentence selection state.
 *
 * Responsible for: maintaining two Sets of selected indices (words and sentences),
 * keeping the sidebar checkbox DOM in sync with state, and dispatching
 * 'selectionchange' CustomEvents so other modules (search, export pane) can react.
 *
 * This module intentionally mixes state with light DOM sync — the sidebar
 * checkboxes are considered a visual projection of the state, not independent UI.
 *
 * Dependencies: data.js (to seed initial state and resolve word objects).
 */
// Responsible for: word and sentence selection state and change notifications via CustomEvents

import { getWordSearchIndex, getSentenceIndex, getAllWords, getWordGroupTitles } from '../data.js';

/** @type {Set<number>} Indices of currently selected vocabulary words. */
let selectedWords = new Set();
/** @type {Set<number>} Indices of currently selected sentences. */
let selectedSentences = new Set();

/**
 * Seeds initial selection: all words and sentences are selected on load.
 * Also marks all sidebar checkboxes and rows as active.
 *
 * Must be called before any other module reads selection state.
 *
 * @returns {void}
 */
export function initSelection() {
  selectedWords     = new Set(getWordSearchIndex().map(w => w.i));
  selectedSentences = new Set(getSentenceIndex().map(s => s.i));
  // Note: sel-circle DOM sync is handled by pageInteractions.js after this runs.
}

/**
 * Dispatches a 'selectionchange' CustomEvent on the window so listeners
 * (search filter, export pane) can update without direct coupling.
 *
 * @returns {void}
 */
function notifySelectionChanged() {
  window.dispatchEvent(new CustomEvent('selectionchange'));
}

/**
 * Adds or removes a word index from the selection and syncs the sidebar checkbox.
 *
 * @param {number}  idx - The word's index in the allWords array.
 * @param {boolean} on  - True to select, false to deselect.
 * @returns {void}
 */
export function setWordSelected(idx, on) {
  if (on) selectedWords.add(idx); else selectedWords.delete(idx);
  notifySelectionChanged();
}

/**
 * Adds or removes a sentence index from the selection and syncs the sidebar checkbox.
 *
 * @param {number}  idx - The sentence's index in the allSentences array.
 * @param {boolean} on  - True to select, false to deselect.
 * @returns {void}
 */
export function setSentenceSelected(idx, on) {
  if (on) selectedSentences.add(idx); else selectedSentences.delete(idx);
  notifySelectionChanged();
}

/**
 * Returns true if the word at the given index is currently selected.
 * @param {number} idx - Word index in the allWords array.
 * @returns {boolean}
 */
export function hasWord(idx) { return selectedWords.has(idx); }

/**
 * Returns true if the sentence at the given index is currently selected.
 * @param {number} idx - Sentence index in the allSentences array.
 * @returns {boolean}
 */
export function hasSentence(idx) { return selectedSentences.has(idx); }

/**
 * Returns an array of all currently selected word indices.
 * @returns {number[]}
 */
export function getSelectedWordIndices() { return [...selectedWords]; }

/**
 * Returns an array of all currently selected sentence indices.
 * @returns {number[]}
 */
export function getSelectedSentenceIndices() { return [...selectedSentences]; }

/**
 * Resolves selected word indices to their full word objects.
 *
 * @returns {object[]} Array of vocabulary word objects for selected indices.
 */
export function getSelectedWordObjects() {
  return [...selectedWords].map(i => getAllWords()[i]).filter(Boolean);
}

/**
 * Suggests a default Anki deck name based on the first selected word's source group.
 * Falls back to 'Hindi::Vocabulary' when nothing is selected.
 *
 * @returns {string} A deck name in the form 'Hindi::<GroupTitle>'.
 */
export function getDefaultDeckName() {
  if (selectedWords.size === 0) return 'Hindi::Vocabulary';
  const firstIndex = [...selectedWords][0];
  const groupTitle = getWordGroupTitles()[firstIndex] ?? 'Vocabulary';
  return `Hindi::${groupTitle}`;
}

/**
 * Syncs a group's master checkbox to reflect whether all its words are selected.
 *
 * Call this after changing individual word selection so the group header
 * checkbox stays consistent.
 *
 * @param {string} groupDate - The data-group attribute value of the target group element.
 * @returns {void}
 */
export function syncGroupCheckbox(groupDate) {
  const groupEl = document.querySelector(`#idx-words .date-group[data-group="${groupDate}"]`);
  if (!groupEl) return;
  const rows         = [...groupEl.querySelectorAll('.idx-row[data-index]')];
  const groupCheckbox = groupEl.querySelector('.group-checkbox');
  if (!groupCheckbox || rows.length === 0) return;
  const allSelected  = rows.every(r => selectedWords.has(parseInt(r.dataset.index)));
  groupCheckbox.classList.toggle('is-on', allSelected);
}
