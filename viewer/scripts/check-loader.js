import assert from 'node:assert/strict';
import { mkdir, mkdtemp, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

import { loadGeneratedData } from '../src/utils/loadGeneratedData.js';

const root = await mkdtemp(join(tmpdir(), 'hindi-viewer-loader-'));
await mkdir(join(root, 'output', 'words'), { recursive: true });
await mkdir(join(root, 'output', 'sentences'), { recursive: true });

await writeFile(
  join(root, 'output', 'words', 'sample_words_batch_01.json'),
  JSON.stringify({
    title: 'Complete Hindi',
    subtitle: 'Chapter 01',
    words: [
      {
        hindi: 'खिड़की',
        romanisation: 'khiṛkī',
        english: 'window, opening',
        forms: [{ hindi: 'खिड़कियाँ', roman: 'khiṛkiyā̃' }],
      },
    ],
  })
);

await writeFile(
  join(root, 'output', 'sentences', 'sample_sentences_batch_01.json'),
  JSON.stringify({
    title: 'Complete Hindi',
    subtitle: 'Chapter 01',
    sentences: [
      {
        hindi: 'क्या यह खिड़की है?',
        romanisation: 'kyā yah khiṛkī hai?',
        english: 'Is this a window?',
        audio: 'audio/sentences/sample/01_window.mp3',
        tokens: [
          { hindi: 'क्या', roman: 'kyā', kind: 'word' },
          { hindi: ' ', roman: ' ', kind: 'space' },
          { hindi: 'यह', roman: 'yah', kind: 'word' },
          { hindi: ' ', roman: ' ', kind: 'space' },
          { hindi: 'खिड़की', roman: 'khiṛkī', kind: 'word' },
          { hindi: ' ', roman: ' ', kind: 'space' },
          { hindi: 'है', roman: 'hai', kind: 'word' },
          { hindi: '?', roman: '?', kind: 'punct' },
        ],
      },
      {
        hindi: 'सब ठीक है?',
        romanisation: 'sab ṭhīk hai?',
        english: 'Everything OK?',
        tokens: [],
      },
    ],
  })
);

await writeFile(join(root, 'output', 'sentences', 'invalid.json'), '{bad json');

const warn = console.warn;
console.warn = () => {};
const payload = await loadGeneratedData(root);
console.warn = warn;

assert.equal(payload.wordFiles.length, 1, 'one valid word file should load');
assert.equal(payload.sentenceFiles.length, 1, 'invalid sentence JSON should be skipped');
assert.equal(payload.allWords.length, 1, 'word card should be normalized into allWords');
assert.equal(payload.allWords[0].groupLabel, 'Complete Hindi Chapter 01');
assert.equal(payload.wordGroups[0].title, 'Complete Hindi Chapter 01');
assert.equal(payload.wordGroupTitles[0], 'Complete Hindi Chapter 01');
assert.equal(payload.hoverData[0].english, 'window', 'hover English should use first comma-separated meaning');

assert.equal(payload.allSentences.length, 2, 'sentences should flatten across groups');
assert.equal(payload.sentenceGroups[0].label, 'Complete Hindi Chapter 01');
assert.equal(payload.sentenceSearchIndex[0].group, 'Complete Hindi Chapter 01');
assert.equal(payload.dataHealth.sentenceTokenReady, 1, 'only one sentence has exact tokens');
assert.equal(payload.dataHealth.sentenceAudioReady, 1, 'only one sentence has audio');
assert.equal(payload.dataHealth.wordAudioReady, 0, 'word without audio should count as missing audio');
assert.equal(payload.qaIssues.length, 3, 'word audio, sentence tokens, and sentence audio should be reported');
assert(payload.qaIssues.some(issue => issue.cardType === 'word' && issue.wordIndex === 0), 'word audio issue should include word jump index');
assert(payload.qaIssues.some(issue => issue.cardType === 'sentence' && issue.sentenceIndex === 1), 'sentence issues should include sentence jump index');
assert(payload.qaIssues.every(issue => issue.groupLabel === 'Complete Hindi Chapter 01'));

console.log('Generated-data loader checks passed.');
