/**
 * Anki Collocations field builder.
 *
 * Responsible for: building the Anki Collocations HTML section field.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki Collocations HTML field
import { esc, aSection } from './utils.js';

/**
 * Renders a single collocation as an HTML row.
 *
 * @param {object} c          - Collocation object with { hindi, roman, english }.
 * @returns {string} HTML div string for the row.
 */
function renderCollocation(c) {
  return `<div style="display:flex;justify-content:space-between;align-items:baseline;padding:.45rem 0;border-bottom:1px solid rgba(30,41,59,.8);">` +
    `<div style="display:flex;flex-direction:column;gap:.15rem;">` +
    `<span style="font-family:'Tiro Devanagari Hindi',serif;font-size:1rem;color:#fbbf24;">${esc(c.hindi)}</span>` +
    `<span style="font-size:.75rem;color:#5eead4;opacity:.6;font-family:monospace;letter-spacing:.05em;">${esc(c.roman)}</span>` +
    `</div>` +
    `<span style="font-size:.85rem;color:#cbd5e1;text-align:right;max-width:55%;">${esc(c.english)}</span>` +
    `</div>`;
}

/**
 * Builds the Anki Collocations field HTML for a word.
 *
 * Returns a styled section with tabular rows, or empty string if absent.
 *
 * @param {object}   word                 - Vocabulary word object.
 * @param {object[]} [word.collocations]  - Array of { hindi, roman, english } objects.
 * @returns {string} HTML section, or empty string if no collocations.
 */
export function buildAnkiCollocations(word) {
  const collocations = word.collocations || [];
  if (!collocations.length) return '';
  const rows = collocations.map(renderCollocation).join('');
  return aSection('Collocations', `<div>${rows}</div>`, '#10b981');
}
