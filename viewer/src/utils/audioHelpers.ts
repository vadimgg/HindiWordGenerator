/**
 * Pure helper functions for resolving audio file paths for words and sentences.
 *
 * Responsible for: resolving audio file paths for words and sentences from their JSON fields.
 *
 * No dependencies on other project modules. No DOM access, no side effects.
 */
// Responsible for: resolving audio file paths for words and sentences from their JSON fields

interface WordAudioLike {
  audio?: string;
  audioBatch?: string;
  hindi?: string;
  romanisation?: string;
}

interface SentenceAudioLike {
  audio?: string;
  audioBatch?: string;
  english?: string;
}

function normaliseAudioSrc(audio?: string): string | undefined {
  if (!audio) return undefined;
  return audio.startsWith('/') ? audio : `/${audio}`;
}

/**
 * Returns the full audio file path for a word.
 * @param audioBatch - vocab filename without extension (e.g. 'hindi_01_vocab_batch_01')
 * @param hindi - word's hindi field
 * @param romanisation - word's romanisation field
 * @returns full path like '/audio/words/hindi_01_vocab_batch_01/लड़का_laṛkā/00_main.mp3'
 */
export function wordAudioSrc(audioBatch: string, hindi: string, romanisation: string): string {
  return `/audio/words/${audioBatch}/${hindi}_${romanisation}/00_main.mp3`;
}

/**
 * Returns the preferred audio file path for a word, using the explicit JSON
 * audio field when present.
 */
export function resolveWordAudioSrc(word: WordAudioLike): string | undefined {
  return normaliseAudioSrc(word.audio);
}

/**
 * Returns the full audio file path for a sentence.
 * @param audioBatch - sentence filename without extension (e.g. 'hindi_01_batch_01')
 * @param english - sentence's english field
 * @returns full path like '/audio/sentences/hindi_01_batch_01/Are_you_Kamala/00_main.mp3'
 */
export function sentenceAudioSrc(audioBatch: string, english: string): string {
  const sanitised = english.replace(/\s+/g, '_').replace(/[?!.,]/g, '');
  return `/audio/sentences/${audioBatch}/${sanitised}/00_main.mp3`;
}

/**
 * Returns the preferred audio file path for a sentence, using the explicit JSON
 * audio field when present.
 */
export function resolveSentenceAudioSrc(sentence: SentenceAudioLike): string | undefined {
  return normaliseAudioSrc(sentence.audio);
}
