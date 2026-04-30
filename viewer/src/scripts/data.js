/**
 * Client-side data accessors for build-time vocabulary data.
 *
 * Responsible for: providing lazy read-only accessors to window.__APP_DATA__,
 * which is populated at build time by index.astro via Astro's define:vars. All
 * getters return empty arrays as a safe fallback so callers never need to null-check.
 *
 * No dependencies on other project modules.
 */
// Responsible for: lazy read-only accessors for window.__APP_DATA__ (set by define:vars in index.astro)

/**
 * Returns the full array of all vocabulary word objects across all loaded JSON files.
 * @returns {object[]}
 */
export const getAllWords = () => window.__APP_DATA__?.allWords ?? [];

/**
 * Returns a sparse array mapping word index → source file title (used for deck name defaults).
 * @returns {string[]}
 */
export const getWordGroupTitles = () => window.__APP_DATA__?.wordGroupTitles ?? [];

/**
 * Returns the compact search index for words: [{ i, h, r, e, d }].
 * i=index, h=hindi, r=romanisation, e=english, d=date_added.
 * @returns {{ i: number, h: string, r: string, e: string, d: string }[]}
 */
export const getWordSearchIndex = () => window.__APP_DATA__?.wordSearchIndex ?? [];

/**
 * Returns the compact search index for sentences: [{ i, h, r, e, d }].
 * @returns {{ i: number, h: string, r: string, e: string, d: string }[]}
 */
export const getSentenceIndex = () => window.__APP_DATA__?.sentenceSearchIndex ?? [];

/**
 * Returns minimal hover data for each word: [{ i, hindi, roman, english, forms }].
 * Used by tooltip.js to build lookup maps without loading full word objects.
 * @returns {{ i: number, hindi: string, roman: string, english: string, forms: {h:string,r:string}[] }[]}
 */
export const getHoverData = () => window.__APP_DATA__?.hoverData ?? [];

/**
 * Returns the full array of all sentence objects across all loaded sentence JSON files.
 * @returns {object[]}
 */
export const getAllSentences = () => window.__APP_DATA__?.allSentences ?? [];
