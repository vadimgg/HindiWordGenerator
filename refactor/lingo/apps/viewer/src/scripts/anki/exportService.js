/**
 * High-level Anki export service.
 *
 * Owns note-type sync, note construction, media upload, incremental sends, and
 * replace-deck sends. UI modules should call these commands rather than
 * coordinating AnkiConnect details themselves.
 */

import { ankiRequest } from './connect.js';
import { ANKI_BACK, ANKI_CSS, ANKI_FIELDS, ANKI_FRONT, ANKI_NOTE_TYPE } from './noteType.js';
import {
  ANKI_SENTENCE_BACK,
  ANKI_SENTENCE_CSS,
  ANKI_SENTENCE_FIELDS,
  ANKI_SENTENCE_FRONT,
  ANKI_SENTENCE_NOTE_TYPE,
} from './sentenceNoteType.js';
import { wordToAnkiFields } from './fields/index.js';
import { sentenceToAnkiFields } from './fields/sentence.js';
import { buildWordTags } from './tagUtils.js';
import { uploadSentenceAudioBatch, uploadWordAudioBatch } from './mediaUploader.js';

async function renameFieldIfPossible(modelName, existingFields, oldFieldName, newFieldName) {
  if (!existingFields.includes(oldFieldName) || existingFields.includes(newFieldName)) {
    return existingFields;
  }

  try {
    await ankiRequest('modelFieldRename', { modelName, oldFieldName, newFieldName });
    return existingFields.map(field => field === oldFieldName ? newFieldName : field);
  } catch {
    return existingFields;
  }
}

async function ensureFields(modelName, fields, renamedFields = {}) {
  let existingFields = await ankiRequest('modelFieldNames', { modelName });
  for (const [oldFieldName, newFieldName] of Object.entries(renamedFields)) {
    existingFields = await renameFieldIfPossible(modelName, existingFields, oldFieldName, newFieldName);
  }

  for (const field of fields) {
    if (!existingFields.includes(field)) {
      await ankiRequest('modelFieldAdd', { modelName, fieldName: field });
    }
  }
}

export async function ensureWordNoteType() {
  const models = await ankiRequest('modelNames', {});
  if (!models.includes(ANKI_NOTE_TYPE)) {
    await ankiRequest('createModel', {
      modelName: ANKI_NOTE_TYPE,
      inOrderFields: ANKI_FIELDS,
      css: ANKI_CSS,
      isCloze: false,
      cardTemplates: [{ Name: 'Recognition', Front: ANKI_FRONT, Back: ANKI_BACK }],
    });
    return;
  }

  await ensureFields(ANKI_NOTE_TYPE, ANKI_FIELDS);
  await ankiRequest('updateModelStyling', { model: { name: ANKI_NOTE_TYPE, css: ANKI_CSS } });
  await ankiRequest('updateModelTemplates', {
    model: {
      name: ANKI_NOTE_TYPE,
      templates: { Recognition: { Front: ANKI_FRONT, Back: ANKI_BACK } },
    },
  });
}

export async function ensureSentenceNoteType() {
  const models = await ankiRequest('modelNames', {});
  if (!models.includes(ANKI_SENTENCE_NOTE_TYPE)) {
    await ankiRequest('createModel', {
      modelName: ANKI_SENTENCE_NOTE_TYPE,
      inOrderFields: ANKI_SENTENCE_FIELDS,
      css: ANKI_SENTENCE_CSS,
      isCloze: false,
      cardTemplates: [{ Name: 'Production', Front: ANKI_SENTENCE_FRONT, Back: ANKI_SENTENCE_BACK }],
    });
    return;
  }

  await ensureFields(ANKI_SENTENCE_NOTE_TYPE, ANKI_SENTENCE_FIELDS, { Chapter: 'Topic' });
  await ankiRequest('updateModelStyling', { model: { name: ANKI_SENTENCE_NOTE_TYPE, css: ANKI_SENTENCE_CSS } });
  await ankiRequest('updateModelTemplates', {
    model: {
      name: ANKI_SENTENCE_NOTE_TYPE,
      templates: { Production: { Front: ANKI_SENTENCE_FRONT, Back: ANKI_SENTENCE_BACK } },
    },
  });
}

function buildWordNotes(words, deckName, allowDuplicate = false) {
  return words.map(word => ({
    deckName,
    modelName: ANKI_NOTE_TYPE,
    fields: wordToAnkiFields(word),
    tags: buildWordTags(word),
    options: { allowDuplicate, duplicateScope: 'deck' },
  }));
}

function buildSentenceNotes(sentences, deckName) {
  return sentences.map(sentence => ({
    deckName,
    modelName: ANKI_SENTENCE_NOTE_TYPE,
    fields: sentenceToAnkiFields(sentence, sentence.groupLabel ?? ''),
    tags: sentence.anki_tags ?? [],
    options: { allowDuplicate: false, duplicateScope: 'deck' },
  }));
}

async function addSkippingDuplicates(notes) {
  const canAdd = await ankiRequest('canAddNotes', { notes });
  const toAdd = notes.filter((_, index) => canAdd[index]);
  const skipped = notes.length - toAdd.length;
  let added = 0;

  if (toAdd.length > 0) {
    const results = await ankiRequest('addNotes', { notes: toAdd });
    added = Array.isArray(results) ? results.filter(result => typeof result === 'number').length : 0;
  }

  return { added, skipped };
}

export async function sendToAnki(words, deckName) {
  await ankiRequest('createDeck', { deck: deckName });
  await ensureWordNoteType();
  await uploadWordAudioBatch(words);
  return addSkippingDuplicates(buildWordNotes(words, deckName, false));
}

export async function sendSentencesToAnki(sentences, deckName) {
  await ankiRequest('createDeck', { deck: deckName });
  await ensureSentenceNoteType();
  await uploadSentenceAudioBatch(sentences);
  return addSkippingDuplicates(buildSentenceNotes(sentences, deckName));
}

export async function overrideDeck(words, deckName) {
  await ankiRequest('createDeck', { deck: deckName });
  await ensureWordNoteType();
  await uploadWordAudioBatch(words);

  const existingIds = await ankiRequest('findNotes', { query: `deck:"${deckName}"` });
  if (existingIds.length > 0) {
    await ankiRequest('deleteNotes', { notes: existingIds });
  }

  const results = await ankiRequest('addNotes', {
    notes: buildWordNotes(words, deckName, true),
  });
  const added = Array.isArray(results) ? results.filter(result => typeof result === 'number').length : 0;
  return { added, deleted: existingIds.length };
}
