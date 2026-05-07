import assert from 'node:assert/strict';

import {
  countSentenceTokenIssues,
  getSentenceTokenIssue,
  hasExactSentenceTokens,
} from '../src/scripts/quality/sentenceTokens.js';

const validSentence = {
  hindi: 'सब ठीक है?',
  romanisation: 'sab ṭhīk hai?',
  words: [
    { hindi: 'सब', roman: 'sab' },
    { hindi: 'ठीक', roman: 'ṭhīk' },
    { hindi: 'है', roman: 'hai' },
  ],
  tokens: [
    { hindi: 'सब', roman: 'sab', kind: 'word', word_index: 0 },
    { hindi: 'ठीक', roman: 'ṭhīk', kind: 'word', word_index: 1 },
    { hindi: 'है', roman: 'hai', kind: 'word', word_index: 2 },
  ],
};

const missingTokenSentence = {
  hindi: 'सब ठीक है?',
  romanisation: 'sab ṭhīk hai?',
  tokens: [],
};

const mismatchedRomanSentence = {
  ...validSentence,
  tokens: [
    { hindi: 'सब', roman: 'sab', kind: 'word', word_index: 0 },
    { hindi: 'ठीक', roman: 'thik', kind: 'word', word_index: 1 },
    { hindi: 'है', roman: 'hai', kind: 'word', word_index: 2 },
  ],
};

const punctuationTokenSentence = {
  ...validSentence,
  tokens: [
    ...validSentence.tokens,
    { hindi: '?', roman: '?', kind: 'punct' },
  ],
};

assert.equal(hasExactSentenceTokens(validSentence), true, 'valid sentence should have word-only tokens');
assert.equal(hasExactSentenceTokens(missingTokenSentence), false, 'missing token array should fail');
assert.equal(hasExactSentenceTokens(mismatchedRomanSentence), false, 'token/word mismatch should fail');
assert.equal(hasExactSentenceTokens(punctuationTokenSentence), false, 'punctuation token should fail');
assert.equal(getSentenceTokenIssue(validSentence), null, 'valid sentence should not return an issue');
assert.equal(getSentenceTokenIssue(missingTokenSentence).code, 'missing-tokens');
assert.equal(getSentenceTokenIssue(mismatchedRomanSentence).code, 'token-word-mismatch');
assert.equal(getSentenceTokenIssue(mismatchedRomanSentence).reason, 'word token does not match word breakdown');
assert.equal(getSentenceTokenIssue(punctuationTokenSentence).code, 'token-count-mismatch');
assert.equal(countSentenceTokenIssues([validSentence, missingTokenSentence, mismatchedRomanSentence, punctuationTokenSentence]), 3);

console.log('Sentence token quality checks passed.');
