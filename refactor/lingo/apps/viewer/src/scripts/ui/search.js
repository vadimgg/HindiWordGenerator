/**
 * Fuzzy search engine and card/row visibility filter.
 *
 * Responsible for: the search algorithm (diacritic-insensitive fuzzy match),
 * DOM caches for word/sentence cards, and the filter function that hides or
 * shows cards based on query and selection state.
 *
 * Search inputs use the [data-search-input] attribute — multiple inputs are
 * kept in sync automatically (e.g. words page filter panel + sentences page filter panel).
 *
 * Does NOT own selection state (state/selection.js) or active tab
 * (state/tabs.js) — it reads those via their public APIs.
 *
 * Dependencies: data.js, state/selection.js, state/tabs.js, utils/stringUtils.js.
 */
// Responsible for: fuzzy search scoring and filtering word/sentence card visibility

import { getWordSearchIndex, getSentenceIndex } from '../data.js';
import { getActiveTab }                         from '../state/tabs.js';
import { norm }                                 from '../utils/stringUtils.js';

/**
 * Scores how well a search needle fuzzy-matches a single haystack string.
 *
 * Returns 1000 for an exact substring hit. For a fuzzy match, characters in
 * the needle must appear in order inside the haystack; consecutive matches
 * earn a streak bonus (×2 per step). Returns -1 if the needle cannot be
 * matched at all.
 *
 * Both inputs are normalised via {@link norm} before comparison so the search
 * is diacritic-insensitive and case-insensitive.
 *
 * @param {string} needle   - The search query string.
 * @param {string} haystack - The candidate string to test against.
 * @returns {number} Score ≥ 0 for a match, -1 for no match.
 */
function fuzzyScore(needle, haystack) {
  needle   = norm(needle);
  haystack = norm(haystack);
  if (!needle) return 0;
  if (haystack.includes(needle)) return 1000;
  let score = 0, hi = 0, streak = 0;
  for (let ni = 0; ni < needle.length; ni++) {
    let found = false;
    while (hi < haystack.length) {
      if (haystack[hi] === needle[ni]) { score += 1 + streak * 2; streak++; hi++; found = true; break; }
      streak = 0; hi++;
    }
    if (!found) return -1;
  }
  return score;
}

/**
 * Returns the best fuzzy score for a needle against all searchable fields
 * of a search-index entry (English, romanisation, Hindi).
 *
 * @param {string} needle - The search query string.
 * @param {{ e: string, r: string, h: string }} entry - A word or sentence search index entry.
 * @returns {number} The highest score across all three fields.
 */
function bestScore(needle, entry) {
  return Math.max(fuzzyScore(needle, entry.e), fuzzyScore(needle, entry.r), fuzzyScore(needle, entry.h));
}

/** @type {Element[]} Ordered list of word card wrapper elements (article[data-word-card]). */
let wordCards = [];
/** @type {Element[]} Ordered list of sentence card wrapper elements. */
let sentCards = [];

/**
 * Initialises search by caching DOM elements and wiring input/clear event listeners.
 *
 * Must be called after DOMContentLoaded. Syncs all [data-search-input] elements
 * to each other so the words and sentences filter panel inputs stay in step.
 * Also listens for 'selectionchange' to re-filter when selection changes.
 *
 * @returns {void}
 */
export function initSearch() {
  wordCards = [...document.querySelectorAll('[data-word-card]')];
  sentCards = [...document.querySelectorAll('[data-sentence-card]')];

  document.querySelectorAll('[data-search-input]').forEach(input => {
    input.addEventListener('input', () => {
      // Keep all search inputs in sync
      document.querySelectorAll('[data-search-input]').forEach(other => {
        if (other !== input) other.value = input.value;
      });
      applyFilter();
    });
  });

  document.querySelectorAll('[data-search-clear]').forEach((btn, i) => {
    btn.addEventListener('click', () => {
      document.querySelectorAll('[data-search-input]').forEach(inp => (inp.value = ''));
      applyFilter();
      document.querySelectorAll('[data-search-input]')[i]?.focus();
    });
  });

  window.addEventListener('selectionchange', applyFilter);
  applyFilter();
}

/**
 * Returns the exported word row DOM map — kept for compatibility with cards.js.
 * Returns an empty object since the sidebar no longer exists.
 *
 * @returns {Record<number, Element>}
 */
export function getWordRowMap() { return {}; }

/**
 * Applies the search filter to the words page: shows/hides cards and
 * updates the count label and empty-state message.
 *
 * In the new design, all words are always rendered; the card is hidden
 * when it does not pass the search filter.
 *
 * @param {string} query - Normalised search query (may be empty).
 * @returns {void}
 */
function filterWords(query) {
  let matchCount = 0;
  getWordSearchIndex().forEach(word => {
    const passes = !query || bestScore(query, word) >= 0;
    const el = wordCards[word.i];
    if (el) el.classList.toggle('hidden', !passes);
    if (passes) matchCount++;
  });

  // Update count in filter panel
  const countEl = document.getElementById('pw-filter-count');
  if (countEl) {
    const total = getWordSearchIndex().length;
    countEl.innerHTML = query
      ? `Showing <strong>${matchCount}</strong> of ${total} words`
      : `Showing <strong>${total}</strong> words`;
  }

  // Show/hide empty groups
  document.querySelectorAll('.card-list').forEach(list => {
    const cards = [...list.querySelectorAll('[data-word-card]')];
    if (cards.length === 0) return;
    const allHidden = cards.every(c => c.classList.contains('hidden'));
    const wrapper = document.getElementById(list.dataset.group ?? '');
    if (wrapper) wrapper.style.display = allHidden ? 'none' : '';
    list.style.display = allHidden ? 'none' : '';
  });

  document.getElementById('words-no-results')?.classList.toggle('hidden', matchCount > 0);
}

/**
 * Applies the search filter to the sentences page: shows/hides cards and
 * updates the count label and empty-state message.
 *
 * @param {string} query - Normalised search query (may be empty).
 * @returns {void}
 */
function filterSentences(query) {
  let matchCount = 0;
  getSentenceIndex().forEach(sentence => {
    const passes = !query || bestScore(query, sentence) >= 0;
    const wrapper = sentCards[sentence.i];
    if (wrapper) wrapper.classList.toggle('hidden', !passes);
    if (passes) matchCount++;
  });

  // Update count in filter panel
  const countEl = document.getElementById('ps-filter-count');
  if (countEl) {
    const total = getSentenceIndex().length;
    countEl.innerHTML = query
      ? `Showing <strong>${matchCount}</strong> of ${total} sentences`
      : `Showing <strong>${total}</strong> sentences`;
  }

  document.getElementById('sentences-no-results')?.classList.toggle('hidden', matchCount > 0);
}

/**
 * Applies the current search query to show/hide cards.
 * Does nothing when the Deliver tab is active.
 *
 * @returns {void}
 */
export function applyFilter() {
  const query = (document.querySelector('[data-search-input]')?.value ?? '').trim();
  document.querySelectorAll('[data-search-clear]').forEach(btn => btn.classList.toggle('hidden', query.length === 0));

  const tab = getActiveTab();
  if (tab === 'deliver') return;
  if (tab === 'words') filterWords(query);
  else filterSentences(query);
}
