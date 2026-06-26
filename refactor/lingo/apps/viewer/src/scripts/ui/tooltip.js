/**
 * Vocab-hint annotation and hover tooltip for example cards.
 *
 * Responsible for: wrapping recognised Hindi and romanised words inside example
 * cards with .vocab-hint spans (on page load), and positioning/populating the
 * floating tooltip when the user hovers over those spans.
 *
 * The card's own main word and its forms are excluded from annotation so that
 * only cross-references to other vocabulary words are highlighted.
 *
 * Dependencies: data.js (hover data), utils/stringUtils.js (norm).
 */
// Responsible for: vocab-hint span annotation in example cards and tooltip positioning

import { getHoverData } from '../data.js';
import { norm }         from '../utils/stringUtils.js';

/**
 * Escapes a string for safe use as an HTML attribute value.
 *
 * @param {string} str - Raw string.
 * @returns {string} String with & and " escaped.
 */
function escAttr(str) {
  return String(str).replace(/&/g, '&amp;').replace(/"/g, '&quot;');
}

/**
 * Wraps Devanagari words in text with .vocab-hint spans if they exist in hindiMap.
 * Words in excludeSet (the card's own word/forms) are left untouched.
 *
 * @param {string}           text       - Raw text content of the Hindi paragraph.
 * @param {Set<string>}      excludeSet - Set of Devanagari strings to skip.
 * @param {Record<string,*>} hindiMap   - Map from Devanagari token → hover entry.
 * @returns {string} Text with matching tokens wrapped in vocab-hint spans.
 */
function annotateHindi(text, excludeSet, hindiMap) {
  return text.replace(/[\u0900-\u097F]+/g, token => {
    if (excludeSet.has(token)) return token;
    const entry = hindiMap[token];
    if (!entry) return token;
    return `<span class="vocab-hint" data-h="${escAttr(entry.hindi)}" data-r="${escAttr(entry.roman)}" data-e="${escAttr(entry.english)}">${token}</span>`;
  });
}

/**
 * Wraps romanised words in text with .vocab-hint spans if they exist in romanMap.
 * Keys in romanMap are normalised (diacritics stripped, lowercase).
 * Words whose normalised form is in excludeSet are left untouched.
 *
 * @param {string}           text       - Raw text content of the romanisation paragraph.
 * @param {Set<string>}      excludeSet - Set of normalised romanisation strings to skip.
 * @param {Record<string,*>} romanMap   - Map from normalised token → hover entry.
 * @returns {string} Text with matching tokens wrapped in vocab-hint spans.
 */
function annotateRoman(text, excludeSet, romanMap) {
  return text.replace(/[a-zA-Zāīūṭḍṇśṣṃḥṛṝḷ]+/g, token => {
    const key = norm(token);
    if (excludeSet.has(key)) return token;
    const entry = romanMap[key];
    if (!entry) return token;
    return `<span class="vocab-hint" data-h="${escAttr(entry.hindi)}" data-r="${escAttr(entry.roman)}" data-e="${escAttr(entry.english)}">${token}</span>`;
  });
}

/**
 * Builds Hindi and romanisation lookup maps from hover data, including inflected forms.
 *
 * @param {object[]} hoverData - Array of hover entries from getHoverData().
 * @returns {{ hindiMap: Record<string,object>, romanMap: Record<string,object> }}
 */
function buildLookupMaps(hoverData) {
  const hindiMap = {};
  const romanMap = {};
  hoverData.forEach(word => {
    hindiMap[word.hindi]       = word;
    romanMap[norm(word.roman)] = word;
    (word.forms || []).forEach(form => {
      if (form.h) hindiMap[form.h]       = word;
      if (form.r) romanMap[norm(form.r)] = word;
    });
  });
  return { hindiMap, romanMap };
}

/**
 * Annotates all example card paragraphs with vocab-hint spans, skipping the card's
 * own word and its inflected forms to avoid self-annotation.
 *
 * @param {object[]}             hoverData - Array of hover entries from getHoverData().
 * @param {Record<string,object>} hindiMap  - Devanagari token → hover entry map.
 * @param {Record<string,object>} romanMap  - Normalised roman token → hover entry map.
 * @returns {void}
 */
function annotateExampleCards(hoverData, hindiMap, romanMap) {
  document.querySelectorAll('[data-example-card]').forEach(container => {
    const cardIndex = parseInt(container.dataset.exampleCard);
    const cardWord  = hoverData[cardIndex];
    if (!cardWord) return;

    const excludeHindi = new Set([cardWord.hindi, ...(cardWord.forms || []).map(f => f.h)]);
    const excludeRoman = new Set([norm(cardWord.roman), ...(cardWord.forms || []).map(f => norm(f.r))]);

    const hindiPara = container.querySelector('[lang="hi"]');
    if (hindiPara) {
      const original  = hindiPara.textContent;
      const annotated = annotateHindi(original, excludeHindi, hindiMap);
      if (annotated !== original) hindiPara.innerHTML = annotated;
    }

    const romanPara = container.querySelector('p.word-roman');
    if (romanPara) {
      const original  = romanPara.textContent;
      const annotated = annotateRoman(original, excludeRoman, romanMap);
      if (annotated !== original) romanPara.innerHTML = annotated;
    }
  });
}

/**
 * Wires the document-level mouseover handler that shows and positions the tooltip.
 *
 * @param {Element} tooltip    - The tooltip container element.
 * @param {Element} tipHindi   - Element to receive the Hindi text.
 * @param {Element} tipRoman   - Element to receive the romanisation text.
 * @param {Element} tipEnglish - Element to receive the English text.
 * @returns {void}
 */
function wireTooltipHover(tooltip, tipHindi, tipRoman, tipEnglish) {
  document.addEventListener('mouseover', e => {
    const hint = e.target.closest('.vocab-hint');
    if (!hint) {
      tooltip.classList.add('hidden');
      tooltip.style.visibility = 'hidden';
      return;
    }
    tipHindi.textContent   = hint.dataset.h;
    tipRoman.textContent   = hint.dataset.r;
    tipEnglish.textContent = hint.dataset.e;

    // Measure after unhiding so we get real dimensions, then snap into view
    tooltip.style.visibility = 'hidden';
    tooltip.classList.remove('hidden');
    requestAnimationFrame(() => {
      const hintRect    = hint.getBoundingClientRect();
      const tooltipRect = tooltip.getBoundingClientRect();
      let top  = hintRect.top - tooltipRect.height - 10;
      let left = hintRect.left + hintRect.width / 2 - tooltipRect.width / 2;
      if (top < 8) top = hintRect.bottom + 10;  // flip below if near top of viewport
      left = Math.max(8, Math.min(left, window.innerWidth - tooltipRect.width - 8));
      tooltip.style.top        = `${top}px`;
      tooltip.style.left       = `${left}px`;
      tooltip.style.visibility = 'visible';
    });
  });
}

/**
 * Initialises tooltip functionality:
 *   1. Builds Hindi and romanisation lookup maps from hover data.
 *   2. Annotates all example card paragraphs with vocab-hint spans.
 *   3. Wires a document-level mouseover handler to show/position the tooltip.
 *
 * Must be called after DOMContentLoaded.
 *
 * @returns {void}
 */
export function initTooltip() {
  const hoverData = getHoverData();
  const { hindiMap, romanMap } = buildLookupMaps(hoverData);
  annotateExampleCards(hoverData, hindiMap, romanMap);

  const tooltip    = document.getElementById('word-tooltip');
  const tipHindi   = document.getElementById('tip-hindi');
  const tipRoman   = document.getElementById('tip-roman');
  const tipEnglish = document.getElementById('tip-english');
  if (!tooltip) return;
  wireTooltipHover(tooltip, tipHindi, tipRoman, tipEnglish);
}
