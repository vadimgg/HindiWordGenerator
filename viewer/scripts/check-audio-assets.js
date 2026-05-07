import assert from 'node:assert/strict';

import {
  audioMediaFilenameFromPath,
  isValidAudioPath,
  resolveSentenceAudioAsset,
  resolveWordAudioAsset,
} from '../src/utils/audioAssets.js';

const word = {
  audio: 'audio/words/sample_batch/01_khiṛkī.mp3',
};
const wordAsset = resolveWordAudioAsset(word);
assert.deepEqual(wordAsset, {
  path: 'audio/words/sample_batch/01_khiṛkī.mp3',
  browserSrc: '/audio/words/sample_batch/01_khiṛkī.mp3',
  mediaFilename: 'words__sample_batch__01_khiṛkī.mp3',
});

const sentence = {
  audio: '/audio/sentences/sample_batch/01_kyā_yah_khiṛkī_hai.mp3',
};
assert.equal(
  resolveSentenceAudioAsset(sentence).mediaFilename,
  'sentences__sample_batch__01_kyā_yah_khiṛkī_hai.mp3',
);

assert.equal(
  audioMediaFilenameFromPath('audio/words/batch/01_test.mp3'),
  'words__batch__01_test.mp3',
);

assert.equal(isValidAudioPath('audio/words/batch/01_test.mp3'), true);
assert.equal(isValidAudioPath('/audio/words/batch/01_test.mp3'), true);
assert.equal(isValidAudioPath('../audio/words/batch/01_test.mp3'), false);
assert.equal(isValidAudioPath('https://example.com/01_test.mp3'), false);
assert.equal(isValidAudioPath('audio/words/batch/01_test.wav'), false);
assert.equal(isValidAudioPath('output/words/batch/01_test.mp3'), false);

console.log('Audio asset contract checks passed.');
