import assert from 'node:assert/strict';

import { ankiRequest, checkAnkiConnect } from '../src/scripts/anki/connect.js';
import { ensureSentenceNoteType, ensureWordNoteType } from '../src/scripts/anki/exportService.js';
import { ANKI_FIELDS, ANKI_NOTE_TYPE } from '../src/scripts/anki/noteType.js';
import { ANKI_SENTENCE_FIELDS, ANKI_SENTENCE_NOTE_TYPE } from '../src/scripts/anki/sentenceNoteType.js';

if (!(await checkAnkiConnect())) {
  console.error('AnkiConnect is not reachable at http://localhost:8765.');
  console.error('Open Anki, enable the AnkiConnect add-on, then rerun: npm run check:ankiconnect');
  process.exit(2);
}

const version = await ankiRequest('version', {});
await ensureWordNoteType();
await ensureSentenceNoteType();

const wordFields = await ankiRequest('modelFieldNames', { modelName: ANKI_NOTE_TYPE });
const sentenceFields = await ankiRequest('modelFieldNames', { modelName: ANKI_SENTENCE_NOTE_TYPE });

for (const field of ANKI_FIELDS) {
  assert(wordFields.includes(field), `${ANKI_NOTE_TYPE} is missing field: ${field}`);
}

for (const field of ANKI_SENTENCE_FIELDS) {
  assert(sentenceFields.includes(field), `${ANKI_SENTENCE_NOTE_TYPE} is missing field: ${field}`);
}

assert(!sentenceFields.includes('Chapter'), `${ANKI_SENTENCE_NOTE_TYPE} still has legacy Chapter field`);

console.log(`AnkiConnect live smoke passed. Version: ${version}`);
