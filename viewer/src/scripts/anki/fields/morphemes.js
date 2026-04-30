/**
 * Anki Morphemes field builder.
 *
 * Responsible for: rendering word morpheme chips (part, roman, meaning, optional origin)
 * as styled HTML for the Anki card back. Returns empty string when the word has no
 * morphemes array.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki Morphemes HTML field
import { aSection, esc } from './utils.js';

/**
 * Builds the Anki Morphemes field HTML for a word.
 *
 * Renders each morpheme as a centred vertical .clang-item card showing:
 * Devanagari part, romanisation, meaning, and optional italic origin label.
 * Section colour is violet-500 (#8b5cf6) to reflect word-structure semantics.
 *
 * @param {object}   word               - Vocabulary word object.
 * @param {object[]} [word.morphemes]   - Array of { part, roman, meaning, origin? }.
 * @param {string}   word.morphemes[].part    - Devanagari morpheme text.
 * @param {string}   word.morphemes[].roman   - Romanised form.
 * @param {string}   word.morphemes[].meaning - English meaning.
 * @param {string}   [word.morphemes[].origin] - Optional language of origin.
 * @returns {string} HTML string, or empty string if no morphemes.
 */
export function buildAnkiMorphemes(word) {
  const morphemes = word.morphemes || [];
  if (morphemes.length === 0) return '';

  const items = morphemes.map(m => {
    const originHtml = m.origin
      ? `<span style="color:#64748b;font-size:1rem;font-style:italic;margin-top:0.25rem;">${esc(m.origin)}</span>`
      : '';
    return (
      `<div class="clang-item">` +
      `<div class="word-row">` +
      `<span class="word-text" lang="hi">${esc(m.part)}</span>` +
      `<span class="word-rom">${esc(m.roman)}</span>` +
      `</div>` +
      `<div class="mean-box"><span>${esc(m.meaning)}</span></div>` +
      originHtml +
      `</div>`
    );
  }).join('');

  return aSection('Morphemes', items, '#8b5cf6');
}
