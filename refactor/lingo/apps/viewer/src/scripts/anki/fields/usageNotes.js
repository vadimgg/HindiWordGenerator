/**
 * Anki Usage Notes field builder.
 *
 * Responsible for: building the Anki UsageNotes HTML section field.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki UsageNotes HTML field
import { esc, aSection } from './utils.js';

/**
 * Builds the Anki UsageNotes field HTML for a word.
 *
 * Returns a styled section with a prose paragraph, or empty string if absent.
 *
 * @param {object} word               - Vocabulary word object.
 * @param {string} [word.usage_notes] - Prose usage note string.
 * @returns {string} HTML section, or empty string if no usage notes.
 */
export function buildAnkiUsageNotes(word) {
  if (!word.usage_notes) return '';
  return aSection('Usage Notes',
    `<p style="font-size:.9rem;color:#cbd5e1;line-height:1.7;">${esc(word.usage_notes)}</p>`,
    '#8b5cf6');
}
