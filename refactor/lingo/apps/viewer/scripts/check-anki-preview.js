import assert from 'node:assert/strict';

import { wordToAnkiFields } from '../src/scripts/anki/fields/index.js';
import { sentenceToAnkiFields } from '../src/scripts/anki/fields/sentence.js';
import { ANKI_FRONT, ANKI_BACK, ANKI_FIELDS } from '../src/scripts/anki/noteType.js';
import {
  ANKI_SENTENCE_FRONT,
  ANKI_SENTENCE_BACK,
  ANKI_SENTENCE_FIELDS,
} from '../src/scripts/anki/sentenceNoteType.js';
import { renderTemplate } from '../src/scripts/anki/renderTemplate.js';
import { sentenceMediaFilename, wordMediaFilename } from '../src/scripts/anki/media.js';

function assertRendered(name, html, expectedSnippets) {
  assert(!html.includes('{{'), `${name} still contains an unresolved template token`);
  for (const snippet of expectedSnippets) {
    assert(html.includes(snippet), `${name} is missing expected snippet: ${snippet}`);
  }
}

const word = {
  english: 'window',
  hindi: 'खिड़की',
  romanisation: 'khiṛkī',
  syllables: 'khiṛ · kī',
  pos: 'noun',
  gender: 'feminine',
  audio: 'audio/words/sample/01_khiṛkī.mp3',
  audioBatch: 'sample_words_batch_01',
  example_sentence: {
    hindi: 'यह खिड़की है।',
    roman: 'yah khiṛkī hai.',
    english: 'This is a window.',
  },
  anki_tags: ['sample'],
};

const wordFields = wordToAnkiFields(word);
assert.deepEqual(Object.keys(wordFields), ANKI_FIELDS, 'word field keys must match ANKI_FIELDS');
assert.equal(wordFields.Audio, `[sound:${wordMediaFilename(word)}]`, 'word audio field must match shared media filename');
assertRendered('word front preview', renderTemplate(ANKI_FRONT, wordFields), ['window']);
assertRendered('word back preview', renderTemplate(ANKI_BACK, wordFields), ['खिड़की', 'khiṛkī']);

const sentence = {
  english: 'Is this a window?',
  hindi: 'क्या यह खिड़की है?',
  romanisation: 'kyā yah khiṛkī hai?',
  literal: 'what this window is',
  register: 'standard',
  audio: 'audio/sentences/sample/01_kyā_yah_khiṛkī_hai.mp3',
  audioBatch: 'sample_sentences_batch_01',
  words: [
    { hindi: 'क्या', roman: 'kyā', meaning: 'question marker' },
    { hindi: 'यह', roman: 'yah', meaning: 'this' },
    { hindi: 'खिड़की', roman: 'khiṛkī', meaning: 'window', gender: 'feminine', number: 'singular' },
    { hindi: 'है', roman: 'hai', meaning: 'is' },
  ],
  anki_tags: ['sample', 'chapter-01'],
};

const sentenceFields = sentenceToAnkiFields(sentence, 'Complete Hindi Chapter 01');
assert.deepEqual(Object.keys(sentenceFields), ANKI_SENTENCE_FIELDS, 'sentence field keys must match ANKI_SENTENCE_FIELDS');
assert.equal(sentenceFields.Topic, 'Complete Hindi Chapter 01', 'sentence topic field must contain the group label');
assert(!('Chapter' in sentenceFields), 'sentence fields must use Topic, not Chapter');
assert.equal(sentenceFields.Audio, `[sound:${sentenceMediaFilename(sentence)}]`, 'sentence audio field must match shared media filename');
assertRendered('sentence front preview', renderTemplate(ANKI_SENTENCE_FRONT, sentenceFields), ['Is this a window?', 'standard']);
assertRendered('sentence back preview', renderTemplate(ANKI_SENTENCE_BACK, sentenceFields), [
  'क्या यह खिड़की है?',
  'Word Breakdown',
  'Complete Hindi Chapter 01',
]);

console.log('Anki preview/export smoke check passed.');
