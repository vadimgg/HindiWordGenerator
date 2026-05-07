/**
 * Anki field assembly for Hindi sentence cards.
 *
 * Responsible for: mapping a sentence object to the complete field object used
 * by both Anki export and the web Anki preview.
 *
 * Dependencies: sentenceBreakdown.js, media.js, utils.js.
 */
// Responsible for: assembling sentenceToAnkiFields() for Hindi Sentence cards

import { buildWordBreakdown } from './sentenceBreakdown.js';
import { sentenceMediaFilename, soundTag } from '../media.js';
import { esc } from './utils.js';

/**
 * Builds the Anki Audio field value for a sentence.
 *
 * @param {object} sentence - Sentence object with optional audio.
 * @returns {string} Anki sound tag or empty string.
 */
export function buildSentenceAudio(sentence) {
  return soundTag(sentenceMediaFilename(sentence));
}

/**
 * Converts a sentence object into the Anki fields object for the Hindi Sentence note type.
 *
 * @param {object} sentence - Sentence object with {hindi, romanisation, english, literal?, register?, words?, anki_tags?}.
 * @param {string} groupLabel - Source/topic label to populate the Topic field.
 * @returns {Record<string, string>} Fields object keyed by sentence Anki field names.
 */
export function sentenceToAnkiFields(sentence, groupLabel) {
  return {
    English:       esc(sentence.english ?? ''),
    Hindi:         esc(sentence.hindi ?? ''),
    Audio:         buildSentenceAudio(sentence),
    Romanisation:  esc(sentence.romanisation ?? ''),
    Literal:       esc(sentence.literal ?? ''),
    Register:      esc(sentence.register ?? ''),
    WordBreakdown: buildWordBreakdown(sentence.words),
    Topic:         esc(groupLabel ?? ''),
    Tags:          (sentence.anki_tags ?? []).join(' '),
  };
}
