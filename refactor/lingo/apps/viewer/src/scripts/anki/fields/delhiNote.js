/**
 * Anki Delhi Note field builder.
 *
 * Responsible for: building the Anki DelhiNote HTML section field.
 *
 * Dependencies: anki/fields/utils.js.
 */
// Responsible for: building the Anki DelhiNote HTML field
import { esc, aSection } from './utils.js';

/**
 * Builds the Anki DelhiNote field HTML for a word.
 *
 * Returns a styled section with a prose paragraph, or empty string if absent.
 *
 * @param {object} word              - Vocabulary word object.
 * @param {string} [word.delhi_note] - Delhi-specific usage note string.
 * @returns {string} HTML section, or empty string if no Delhi note.
 */
export function buildAnkiDelhiNote(word) {
  if (!word.delhi_note) return '';
  return aSection('Delhi Usage',
    `<p style="font-size:.9rem;color:#cbd5e1;line-height:1.7;">${esc(word.delhi_note)}</p>`,
    '#fbbf24');
}
