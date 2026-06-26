/**
 * Backward-compatible Anki export facade.
 *
 * New orchestration lives in exportService.js; this file keeps existing UI
 * imports stable while the caller modules are simplified.
 */

export {
  ensureSentenceNoteType,
  overrideDeck,
  sendSentencesToAnki,
  sendToAnki,
} from './exportService.js';

export {
  uploadSentenceAudio,
  uploadWordAudio,
} from './mediaUploader.js';
