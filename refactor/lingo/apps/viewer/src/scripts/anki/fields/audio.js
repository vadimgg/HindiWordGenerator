/**
 * Anki Audio field builder — [sound:] syntax for word pronunciation.
 *
 * Responsible for: building the Anki Audio field value using AnkiConnect's
 * [sound:] tag syntax. Anki media files are stored flat, so the filename is
 * derived from the canonical JSON audio path.
 *
 * Dependencies: anki/media.js.
 */
// Responsible for: building the Anki Audio field with [sound:] syntax for word pronunciation

import { soundTag, wordMediaFilename } from '../media.js';

/**
 * Builds the Anki Audio field value for a vocabulary word.
 *
 * Returns an empty string if the word has no valid audio path.
 *
 * @param {object} word - Vocabulary word object.
 * @param {string} [word.audio] - Project-relative audio path.
 * @returns {string} Anki sound tag or empty string.
 */
export function buildAnkiAudio(word) {
  return soundTag(wordMediaFilename(word));
}
