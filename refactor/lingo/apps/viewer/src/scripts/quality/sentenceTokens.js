/**
 * Sentence token quality checks.
 *
 * Responsible for: validating whether sentence token arrays contain one clean
 * word token for each learner-facing word breakdown entry.
 *
 * No DOM access and no dependencies on other project modules.
 */
// Responsible for: shared sentence word-token contract checks

/**
 * Returns a structured token issue, or null when tokens match words exactly.
 *
 * @param {object} sentence - Sentence card data.
 * @returns {{code:string, reason:string, expected?:object, actual?:object}|null}
 */
export function getSentenceTokenIssue(sentence) {
  if (!Array.isArray(sentence?.tokens) || sentence.tokens.length === 0) {
    return { code: 'missing-tokens', reason: 'missing word tokens' };
  }

  if (!Array.isArray(sentence?.words) || sentence.words.length === 0) {
    return { code: 'missing-words', reason: 'missing word breakdown' };
  }

  const wordById = new Map();
  let hasWordIds = false;
  for (const word of sentence.words) {
    if (typeof word.id === 'string' && word.id.trim() !== '') {
      hasWordIds = true;
      wordById.set(word.id, word);
    }
  }

  if (!hasWordIds && sentence.tokens.length !== sentence.words.length) {
    return {
      code: 'token-count-mismatch',
      reason: 'word token count does not match word breakdown',
      expected: { count: sentence.words.length },
      actual: { count: sentence.tokens.length },
    };
  }

  for (let index = 0; index < sentence.tokens.length; index += 1) {
    const token = sentence.tokens[index];
    if (token.kind !== 'word') {
      return { code: 'non-word-token', reason: 'tokens must contain words only', actual: token };
    }
    const word = typeof token.word_id === 'string'
      ? wordById.get(token.word_id)
      : sentence.words[index];
    if (!word) {
      return {
        code: 'word-link-missing',
        reason: 'word token does not reference a word breakdown entry',
        actual: { word_id: token.word_id, word_index: token.word_index },
      };
    }
    if (!token.word_id && token.word_index !== index) {
      return {
        code: 'word-index-mismatch',
        reason: 'word token index does not match token position',
        expected: { word_index: index },
        actual: { word_index: token.word_index },
      };
    }
    if (token.hindi !== word.hindi || token.roman !== word.roman) {
      return {
        code: 'token-word-mismatch',
        reason: 'word token does not match word breakdown',
        expected: { hindi: word.hindi, roman: word.roman },
        actual: { hindi: token.hindi, roman: token.roman },
      };
    }
  }

  return null;
}

/**
 * Returns true when sentence.tokens contains one clean word token per words entry.
 *
 * @param {object} sentence - Sentence card data.
 * @returns {boolean}
 */
export function hasExactSentenceTokens(sentence) {
  return getSentenceTokenIssue(sentence) === null;
}

/**
 * Counts sentences whose tokens are absent or do not match the word breakdown.
 *
 * @param {object[]} sentences - Sentence card data.
 * @returns {number}
 */
export function countSentenceTokenIssues(sentences) {
  return sentences.filter(sentence => !hasExactSentenceTokens(sentence)).length;
}
