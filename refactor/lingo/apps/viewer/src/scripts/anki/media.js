import { resolveSentenceAudioAsset, resolveWordAudioAsset } from '../../utils/audioAssets.js';

/**
 * Builds the flat Anki media filename for a word audio file.
 *
 * @param {object} word - Word object with audioBatch, hindi, romanisation.
 * @returns {string} Media filename, or an empty string when required data is missing.
 */
export function wordMediaFilename(word) {
  return resolveWordAudioAsset(word)?.mediaFilename ?? '';
}

/**
 * Builds the flat Anki media filename for a sentence audio file.
 *
 * @param {object} sentence - Sentence object with audioBatch and english.
 * @returns {string} Media filename, or an empty string when required data is missing.
 */
export function sentenceMediaFilename(sentence) {
  return resolveSentenceAudioAsset(sentence)?.mediaFilename ?? '';
}

/**
 * Wraps a media filename in Anki's sound tag syntax.
 *
 * @param {string} filename - Flat Anki media filename.
 * @returns {string} Anki sound tag, or empty string when filename is empty.
 */
export function soundTag(filename) {
  return filename ? `[sound:${filename}]` : '';
}
