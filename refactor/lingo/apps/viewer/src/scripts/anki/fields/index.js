/**
 * Anki field assembly — combines all per-field HTML builders into one object.
 *
 * Responsible for: providing a single wordToAnkiFields() function that maps a
 * vocabulary word to a complete Anki fields object. Each field is produced by
 * a dedicated builder module; this file only wires them together.
 *
 * Dependencies: all files in anki/fields/, noteType.js (ANKI_FIELDS).
 */
// Responsible for: assembling wordToAnkiFields() from all individual field builders
import { ANKI_FIELDS }           from '../noteType.js';
import { buildAnkiCategory }     from './category.js';
import { buildAnkiGender }       from './gender.js';
import { buildAnkiTransitivity } from './transitivity.js';
import { buildAnkiSyllables }    from './syllables.js';
import { buildAnkiForms }        from './forms.js';
import { buildAnkiUsageNotes }   from './usageNotes.js';
import { buildAnkiRelatedWords } from './relatedWords.js';
import { buildAnkiMorphemes }    from './morphemes.js';
import { buildAnkiSoundAlikes }    from './soundAlikes.js';
import { buildAnkiExampleSentence } from './exampleSentence.js';
import { buildAnkiDelhiNote }      from './delhiNote.js';
import { buildAnkiEtymology }      from './etymology.js';
import { buildAnkiAudio }         from './audio.js';
import { esc }                   from './utils.js';

/**
 * Converts a vocabulary word object into a complete Anki fields object.
 *
 * Returns a plain object keyed by field name (matching ANKI_FIELDS in noteType.js).
 * Each value is an HTML string safe for Anki's HTML card renderer.
 *
 * @param {object} word - Vocabulary word object from the vocab JSON files.
 * @returns {Record<string, string>} Map of Anki field name → HTML string.
 */
export function wordToAnkiFields(word) {
  return {
    English:      esc(word.english || ''),
    Hindi:        esc(word.hindi || ''),
    Audio:        buildAnkiAudio(word),
    Romanisation: esc(word.romanisation || ''),
    Syllables:    buildAnkiSyllables(word),
    Category:     buildAnkiCategory(word),
    Gender:       buildAnkiGender(word),
    Transitivity: buildAnkiTransitivity(word),
    Forms:            buildAnkiForms(word),
    ExampleSentence:  buildAnkiExampleSentence(word),
    ExampleEnglish:   esc(word.example_sentence?.english ?? ''),
    UsageNotes:       buildAnkiUsageNotes(word),
    RelatedWords:     buildAnkiRelatedWords(word),
    Morphemes:        buildAnkiMorphemes(word),
    SoundAlikes:      buildAnkiSoundAlikes(word),
    DelhiNote:        buildAnkiDelhiNote(word),
    Etymology:        buildAnkiEtymology(word),
  };
}

// Sync assertion: wordToAnkiFields keys must exactly match ANKI_FIELDS (same order).
const _actualKeys = Object.keys(wordToAnkiFields({}));
if (JSON.stringify(_actualKeys) !== JSON.stringify(ANKI_FIELDS)) {
  throw new Error(
    `wordToAnkiFields/ANKI_FIELDS mismatch.\n  ANKI_FIELDS: ${ANKI_FIELDS.join(', ')}\n  wordToAnkiFields: ${_actualKeys.join(', ')}`
  );
}
