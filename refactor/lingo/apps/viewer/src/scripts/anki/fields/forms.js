/**
 * Anki Forms field builder.
 *
 * Responsible for: rendering inflected forms as styled chips.
 * Returns empty string when the word has no forms.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki Forms HTML field (inflected form chips)
import { esc, ankiChipStyle } from './utils.js';

/**
 * Builds the Anki Forms field HTML for a word.
 *
 * Renders form chips (label + Devanagari + romanisation) for each inflected form.
 *
 * @param {object}   word          - Vocabulary word object.
 * @param {object[]} [word.forms]  - Array of { label, hindi, roman } form objects.
 * @returns {string} HTML string, or empty string if no forms.
 */
export function buildAnkiForms(word) {
  const forms = word.forms || [];
  if (forms.length === 0) return '';

  const chips = forms.map(form => {
    const chipStyle = ankiChipStyle(form.label);
    return `<span style="display:inline-flex;align-items:baseline;gap:.4rem;padding:.25rem .7rem;border-radius:9999px;font-size:.75rem;font-weight:500;${chipStyle}">` +
      `<span style="font-family:'Barlow Condensed',sans-serif;font-size:.62rem;text-transform:uppercase;letter-spacing:.08em;">${esc(form.label)}</span>` +
      `<span style="font-family:'Tiro Devanagari Hindi',serif;font-size:1.1em;color:#fbbf24;">${esc(form.hindi)}</span>` +
      `<span style="color:#6b7280">·</span>` +
      `<span style="color:#5eead4;">${esc(form.roman)}</span>` +
      `</span>`;
  }).join('');
  return `<div style="display:flex;flex-wrap:wrap;gap:.5rem;">${chips}</div>`;
}
