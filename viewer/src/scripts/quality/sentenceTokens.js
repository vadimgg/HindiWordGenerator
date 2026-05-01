/**
 * Sentence token quality checks.
 *
 * Responsible for: validating whether sentence token arrays exactly reconstruct
 * the displayed Hindi and romanisation strings.
 *
 * No DOM access and no dependencies on other project modules.
 */
// Responsible for: shared sentence token reconstruction checks

function reconstructedTokens(sentence) {
  return {
    hindi: sentence.tokens.map(token => token.hindi ?? '').join(''),
    romanisation: sentence.tokens.map(token => token.roman ?? '').join(''),
  };
}

/**
 * Returns a structured token reconstruction issue, or null when tokens are exact.
 *
 * @param {object} sentence - Sentence card data.
 * @returns {{code:string, reason:string, expected?:object, actual?:object}|null}
 */
export function getSentenceTokenIssue(sentence) {
  if (!Array.isArray(sentence?.tokens) || sentence.tokens.length === 0) {
    return { code: 'missing-tokens', reason: 'missing exact tokens' };
  }

  const actual = reconstructedTokens(sentence);
  const expected = {
    hindi: sentence.hindi ?? '',
    romanisation: sentence.romanisation ?? '',
  };
  const hindiMatches = actual.hindi === expected.hindi;
  const romanMatches = actual.romanisation === expected.romanisation;

  if (hindiMatches && romanMatches) return null;
  if (!hindiMatches && !romanMatches) {
    return { code: 'token-mismatch', reason: 'Hindi and romanisation tokens mismatch', expected, actual };
  }
  if (!hindiMatches) {
    return { code: 'hindi-token-mismatch', reason: 'Hindi tokens mismatch', expected, actual };
  }
  return { code: 'roman-token-mismatch', reason: 'romanisation tokens mismatch', expected, actual };
}

/**
 * Returns true when sentence.tokens exactly reconstructs both Hindi and romanisation.
 *
 * @param {object} sentence - Sentence card data.
 * @returns {boolean}
 */
export function hasExactSentenceTokens(sentence) {
  return getSentenceTokenIssue(sentence) === null;
}

/**
 * Counts sentences whose tokens are absent or do not reconstruct the source strings.
 *
 * @param {object[]} sentences - Sentence card data.
 * @returns {number}
 */
export function countSentenceTokenIssues(sentences) {
  return sentences.filter(sentence => !hasExactSentenceTokens(sentence)).length;
}
