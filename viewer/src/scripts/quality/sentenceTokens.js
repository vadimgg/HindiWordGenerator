/**
 * Sentence token quality checks.
 *
 * Responsible for: validating whether sentence token arrays exactly reconstruct
 * the displayed Hindi and romanisation strings.
 *
 * No DOM access and no dependencies on other project modules.
 */
// Responsible for: shared sentence token reconstruction checks

/**
 * Returns true when sentence.tokens exactly reconstructs both Hindi and romanisation.
 *
 * @param {object} sentence - Sentence card data.
 * @returns {boolean}
 */
export function hasExactSentenceTokens(sentence) {
  if (!Array.isArray(sentence?.tokens) || sentence.tokens.length === 0) return false;
  const joinedHindi = sentence.tokens.map(token => token.hindi ?? '').join('');
  const joinedRoman = sentence.tokens.map(token => token.roman ?? '').join('');
  return joinedHindi === (sentence.hindi ?? '') && joinedRoman === (sentence.romanisation ?? '');
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
