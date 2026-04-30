/**
 * Shared TypeScript type definitions for the Hindi Vocabulary app.
 *
 * Responsible for: declaring all interfaces and types used in Astro component
 * props, utility functions, and client-side scripts. Centralising them here
 * ensures that the shape of vocabulary JSON data is documented in one place.
 *
 * No runtime code — types only. No dependencies on other project modules.
 */
// Responsible for: shared TypeScript interfaces used across Astro components and client scripts

export interface Form { label: string; hindi: string; roman: string; }
export interface Morpheme { part: string; roman: string; meaning: string; origin?: string; }
export interface WordGroup { id: string; title: string; words: { w: Word; i: number }[]; }
export interface SearchEntry { i: number; h: string; r: string; e: string; d: string; }
export interface HoverEntry { i: number; hindi: string; roman: string; english: string; forms: { h: string; r: string }[]; }
export interface Collocation { hindi: string; roman: string; english: string; }
export interface RelatedWord { hindi: string; roman: string; english: string; }
export interface EtymologyStage { stage: string; form: string; roman: string; meaning: string; }
export interface SoundAlike { part: string; association: string; roman?: string; language?: string; note?: string; }

export interface ExampleSentenceToken {
  hindi: string;
  roman: string;
  meaning: string;
}

export interface ExampleSentence {
  hindi: string;
  roman: string;
  english: string;
  breakdown?: ExampleSentenceToken[];
}

/**
 * A single vocabulary word as stored in the vocab JSON files.
 *
 * Required fields: hindi, romanisation, english.
 * All other fields are optional — their presence depends on how deeply the
 * word has been annotated.
 */
export interface Word {
  /** Devanagari script form of the word. */
  hindi: string;
  /** Romanised transliteration using diacritics. */
  romanisation: string;
  /** Primary English meaning or meanings. */
  english: string;
  /** Part of speech: 'noun', 'verb', 'adjective', etc. */
  pos?: string;
  /** Grammatical gender: 'masculine', 'feminine', or 'both'. */
  gender?: 'masculine' | 'feminine' | 'both';
  /** Verb transitivity: 'transitive' or 'intransitive'. */
  transitivity?: 'transitive' | 'intransitive';
  /** Syllable breakdown string, e.g. "laṛ · kā". */
  syllables?: string;
  /** Inflected forms (feminine, plural, oblique, etc.). */
  forms?: Form[];
  /** Prose usage note shown in the Usage Notes section. */
  usage_notes?: string;
  /** Related words with Hindi, romanisation, and English gloss. */
  related_words?: RelatedWord[];
  /** Sound-alike memory hooks linking the word to familiar sounds. */
  sound_alikes?: SoundAlike[];
  /** Ordered etymology stages from oldest ancestor to modern Hindi. */
  etymology_journey?: EtymologyStage[];
  /** Prose note about the word's origin, shown beneath the etymology chain. */
  origin_note?: string;
  /** Delhi-specific usage note describing how the word is used in everyday Delhi speech. */
  delhi_note?: string;
  /** Example sentence illustrating the word in context. */
  example_sentence?: ExampleSentence;
  /** Anki tags to apply when exporting this word as a flashcard. */
  anki_tags?: string[];
  /** Relative public audio path, if available in the source JSON. */
  audio?: string;
  /** Vocab batch filename without extension, used to resolve audio paths. */
  audioBatch?: string;
}

export interface SentenceWord {
  hindi: string;
  roman: string;
  meaning: string;
  note?: string;
  gender?: 'masculine' | 'feminine';
  number?: 'singular' | 'plural';
}

export interface SentenceToken {
  hindi: string;
  roman: string;
  kind: 'word' | 'punct';
  word_index?: number;
}

export interface Sentence {
  hindi: string;
  romanisation: string;
  english: string;
  literal?: string;
  register?: 'formal' | 'neutral' | 'standard' | 'casual' | 'colloquial';
  tokens?: SentenceToken[];
  words?: SentenceWord[];
  anki_tags?: string[];
  /** Relative public audio path, if available in the source JSON. */
  audio?: string;
  /** Sentence batch filename without extension, used to resolve audio paths. */
  audioBatch?: string;
}

export interface SentenceChapter {
  chapter: string;
  sentences: Sentence[];
}
