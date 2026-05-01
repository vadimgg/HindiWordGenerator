/**
 * Anki export orchestration — deck creation, note type sync, and card addition.
 *
 * Responsible for: coordinating the full export pipeline from a word array to
 * Anki cards. Handles two modes — incremental (skip duplicates) and override
 * (wipe deck then re-add). Does NOT build HTML field content or generate tags;
 * those concerns live in fields/index.js and tagUtils.js respectively.
 *
 * Dependencies: anki/connect.js, anki/noteType.js, anki/sentenceNoteType.js,
 *               anki/fields/index.js, anki/fields/sentenceBreakdown.js,
 *               anki/fields/utils.js, anki/tagUtils.js.
 */
// Responsible for: orchestrating Anki deck creation, note type verification, and card addition
import { ankiRequest }                                           from './connect.js';
import { ANKI_NOTE_TYPE, ANKI_FIELDS, ANKI_CSS, ANKI_FRONT, ANKI_BACK } from './noteType.js';
import {
  ANKI_SENTENCE_NOTE_TYPE, ANKI_SENTENCE_FIELDS,
  ANKI_SENTENCE_CSS, ANKI_SENTENCE_FRONT, ANKI_SENTENCE_BACK,
} from './sentenceNoteType.js';
import { wordToAnkiFields }                                      from './fields/index.js';
import { sentenceToAnkiFields }                                  from './fields/sentence.js';
import { buildWordTags }                                         from './tagUtils.js';
import { sentenceMediaFilename, wordMediaFilename }               from './media.js';
import { resolveSentenceAudioSrc, resolveWordAudioSrc }          from '../../utils/audioHelpers.ts';

/**
 * Creates the note type if it does not exist, or syncs its CSS and templates if it does.
 * Called before every export so design changes on the website propagate to Anki on re-export.
 *
 * @returns {Promise<void>}
 */
async function ensureNoteType() {
  const models = await ankiRequest('modelNames', {});
  if (!models.includes(ANKI_NOTE_TYPE)) {
    await ankiRequest('createModel', {
      modelName: ANKI_NOTE_TYPE, inOrderFields: ANKI_FIELDS,
      css: ANKI_CSS, isCloze: false,
      cardTemplates: [{ Name: 'Recognition', Front: ANKI_FRONT, Back: ANKI_BACK }],
    });
  } else {
    // Add any fields that exist in ANKI_FIELDS but are missing from the note type
    // (e.g. Morphemes added after the note type was first created in Anki).
    // Must run BEFORE updateModelTemplates — Anki validates field names immediately
    // when templates are updated and throws if a referenced field does not exist yet.
    const existingFields = await ankiRequest('modelFieldNames', { modelName: ANKI_NOTE_TYPE });
    for (const field of ANKI_FIELDS) {
      if (!existingFields.includes(field)) {
        await ankiRequest('modelFieldAdd', { modelName: ANKI_NOTE_TYPE, fieldName: field });
      }
    }
    // Always sync CSS and templates so design changes take effect on re-export
    await ankiRequest('updateModelStyling', { model: { name: ANKI_NOTE_TYPE, css: ANKI_CSS } });
    await ankiRequest('updateModelTemplates', {
      model: {
        name: ANKI_NOTE_TYPE,
        templates: { Recognition: { Front: ANKI_FRONT, Back: ANKI_BACK } },
      },
    });
  }
}

/**
 * Converts an array of vocabulary words into Anki note objects ready for the API.
 *
 * @param {object[]} words          - Vocabulary word objects.
 * @param {string}   deckName       - Target Anki deck name.
 * @param {boolean}  [allowDuplicate=false] - Whether Anki should accept duplicate notes.
 * @returns {object[]} Array of Anki note objects.
 */
function buildNotes(words, deckName, allowDuplicate = false) {
  return words.map(word => ({
    deckName,
    modelName: ANKI_NOTE_TYPE,
    fields:    wordToAnkiFields(word),
    tags:      buildWordTags(word),
    options:   { allowDuplicate, duplicateScope: 'deck' },
  }));
}

/**
 * Creates the Hindi Sentence note type if it does not exist, or syncs its CSS and
 * templates if it does. Called before every sentence export.
 *
 * @returns {Promise<void>}
 */
export async function ensureSentenceNoteType() {
  const models = await ankiRequest('modelNames', {});
  if (!models.includes(ANKI_SENTENCE_NOTE_TYPE)) {
    await ankiRequest('createModel', {
      modelName: ANKI_SENTENCE_NOTE_TYPE, inOrderFields: ANKI_SENTENCE_FIELDS,
      css: ANKI_SENTENCE_CSS, isCloze: false,
      cardTemplates: [{ Name: 'Production', Front: ANKI_SENTENCE_FRONT, Back: ANKI_SENTENCE_BACK }],
    });
  } else {
    // Add any fields that exist in ANKI_SENTENCE_FIELDS but are missing from the note type
    // (e.g. Audio added after the note type was first created in Anki).
    // Must run BEFORE updateModelTemplates — Anki validates field names immediately.
    const existingFields = await ankiRequest('modelFieldNames', { modelName: ANKI_SENTENCE_NOTE_TYPE });
    for (const field of ANKI_SENTENCE_FIELDS) {
      if (!existingFields.includes(field)) {
        await ankiRequest('modelFieldAdd', { modelName: ANKI_SENTENCE_NOTE_TYPE, fieldName: field });
      }
    }
    // Always sync CSS and templates so design changes take effect on re-export
    await ankiRequest('updateModelStyling', { model: { name: ANKI_SENTENCE_NOTE_TYPE, css: ANKI_SENTENCE_CSS } });
    await ankiRequest('updateModelTemplates', {
      model: {
        name: ANKI_SENTENCE_NOTE_TYPE,
        templates: { Production: { Front: ANKI_SENTENCE_FRONT, Back: ANKI_SENTENCE_BACK } },
      },
    });
  }
}

export { sentenceToAnkiFields };

/**
 * Fetches a sentence's audio file and uploads it to Anki's media store via
 * AnkiConnect's storeMediaFile action. Returns silently on failure — audio is optional.
 *
 * The filename is namespaced (batch__sanitisedEnglish__sentence.mp3) so it is
 * unique in Anki's flat media folder.
 *
 * @param {object} sentence - Sentence object with audioBatch and english fields.
 * @returns {Promise<void>}
 */
export async function uploadSentenceAudio(sentence) {
  const path = resolveSentenceAudioSrc(sentence);
  const filename = sentenceMediaFilename(sentence);
  if (!path || !filename) return;
  try {
    const response = await fetch(path);
    if (!response.ok) return;
    const buffer = await response.arrayBuffer();
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    const base64 = btoa(binary);
    await ankiRequest('storeMediaFile', { filename, data: base64 });
  } catch {
    // Audio upload is optional — silently skip on any failure
  }
}

/**
 * Fetches a word's normal audio file and uploads it to Anki's media store via
 * AnkiConnect's storeMediaFile action. Returns silently on failure — audio is optional.
 *
 * The filename is namespaced (batch__hindi_roman__word.mp3) so it is unique in
 * Anki's flat media folder. Anki resolves [sound:filename] against this store.
 *
 * @param {object} word - Vocabulary word object with audioBatch, hindi, romanisation.
 * @returns {Promise<void>}
 */
async function uploadWordAudio(word) {
  const path = resolveWordAudioSrc(word);
  const filename = wordMediaFilename(word);
  if (!path || !filename) return;
  try {
    const response = await fetch(path);
    if (!response.ok) return;
    const buffer = await response.arrayBuffer();
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    const base64 = btoa(binary);
    await ankiRequest('storeMediaFile', { filename, data: base64 });
  } catch {
    // Audio upload is optional — silently skip on any failure
  }
}

/**
 * Exports selected words to Anki, skipping cards that already exist in the deck.
 *
 * @param {object[]} words    - Vocabulary word objects to export.
 * @param {string}   deckName - Target Anki deck name (created if it does not exist).
 * @returns {Promise<{ added: number, skipped: number }>} Counts of added and skipped cards.
 */
export async function sendToAnki(words, deckName) {
  await ankiRequest('createDeck', { deck: deckName });
  await ensureNoteType();
  // Upload audio files before adding notes (failures are silently ignored)
  await Promise.all(words.map(w => uploadWordAudio(w)));
  const notes   = buildNotes(words, deckName, false);
  const canAdd  = await ankiRequest('canAddNotes', { notes });
  const toAdd   = notes.filter((_, i) => canAdd[i]);
  const skipped = notes.length - toAdd.length;
  let added = 0;
  if (toAdd.length > 0) {
    const results = await ankiRequest('addNotes', { notes: toAdd });
    // Only count numeric IDs — newer AnkiConnect returns error strings for failed notes
    added = Array.isArray(results) ? results.filter(r => typeof r === 'number').length : 0;
  }
  return { added, skipped };
}

/**
 * Replaces all cards in the target deck with the given words.
 *
 * Deletes every existing note in the deck first, then adds all selected words
 * with duplicate checking disabled (since the deck was just cleared).
 *
 * @param {object[]} words    - Vocabulary word objects to export.
 * @param {string}   deckName - Target Anki deck name.
 * @returns {Promise<{ added: number, deleted: number }>} Counts of added and deleted cards.
 */
export async function overrideDeck(words, deckName) {
  await ankiRequest('createDeck', { deck: deckName });
  await ensureNoteType();
  // Upload audio files before replacing notes (failures are silently ignored)
  await Promise.all(words.map(w => uploadWordAudio(w)));
  const existingIds = await ankiRequest('findNotes', { query: `deck:"${deckName}"` });
  if (existingIds.length > 0) await ankiRequest('deleteNotes', { notes: existingIds });
  // Duplicates allowed since we just cleared the deck
  const notes   = buildNotes(words, deckName, true);
  const results = await ankiRequest('addNotes', { notes });
  const added   = Array.isArray(results) ? results.filter(r => typeof r === 'number').length : 0;
  return { added, deleted: existingIds.length };
}
