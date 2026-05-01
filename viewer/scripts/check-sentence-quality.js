import assert from 'node:assert/strict';

import {
  countSentenceTokenIssues,
  getSentenceTokenIssue,
  hasExactSentenceTokens,
} from '../src/scripts/quality/sentenceTokens.js';

const validSentence = {
  hindi: 'सब ठीक है?',
  romanisation: 'sab ṭhīk hai?',
  tokens: [
    { hindi: 'सब', roman: 'sab', kind: 'word', word_index: 0 },
    { hindi: ' ', roman: ' ', kind: 'space' },
    { hindi: 'ठीक', roman: 'ṭhīk', kind: 'word', word_index: 1 },
    { hindi: ' ', roman: ' ', kind: 'space' },
    { hindi: 'है', roman: 'hai', kind: 'word', word_index: 2 },
    { hindi: '?', roman: '?', kind: 'punct' },
  ],
};

const missingTokenSentence = {
  hindi: 'सब ठीक है?',
  romanisation: 'sab ṭhīk hai?',
  tokens: [],
};

const mismatchedRomanSentence = {
  ...validSentence,
  romanisation: 'sab thik hai?',
};

assert.equal(hasExactSentenceTokens(validSentence), true, 'valid sentence should reconstruct exactly');
assert.equal(hasExactSentenceTokens(missingTokenSentence), false, 'missing token array should fail');
assert.equal(hasExactSentenceTokens(mismatchedRomanSentence), false, 'romanisation mismatch should fail');
assert.equal(getSentenceTokenIssue(validSentence), null, 'valid sentence should not return an issue');
assert.equal(getSentenceTokenIssue(missingTokenSentence).code, 'missing-tokens');
assert.equal(getSentenceTokenIssue(mismatchedRomanSentence).code, 'roman-token-mismatch');
assert.equal(getSentenceTokenIssue(mismatchedRomanSentence).reason, 'romanisation tokens mismatch');
assert.equal(countSentenceTokenIssues([validSentence, missingTokenSentence, mismatchedRomanSentence]), 2);

console.log('Sentence token quality checks passed.');
