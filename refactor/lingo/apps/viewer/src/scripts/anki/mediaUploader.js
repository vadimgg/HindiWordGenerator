/**
 * Anki media upload service.
 *
 * Fetches browser-served audio assets and stores them in Anki's flat media
 * folder. Audio is optional, so upload failures are intentionally non-fatal.
 */

import { resolveSentenceAudioAsset, resolveWordAudioAsset } from '../../utils/audioAssets.js';
import { ankiRequest } from './connect.js';
import { sentenceMediaFilename, wordMediaFilename } from './media.js';

async function assetToBase64(browserSrc) {
  const response = await fetch(browserSrc);
  if (!response.ok) return '';

  const buffer = await response.arrayBuffer();
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

async function storeAudio(asset, filename) {
  if (!asset || !filename) return;
  try {
    const data = await assetToBase64(asset.browserSrc);
    if (!data) return;
    await ankiRequest('storeMediaFile', { filename, data });
  } catch {
    // Audio is optional for export, so failed uploads should not block cards.
  }
}

export function uploadWordAudio(word) {
  return storeAudio(resolveWordAudioAsset(word), wordMediaFilename(word));
}

export function uploadSentenceAudio(sentence) {
  return storeAudio(resolveSentenceAudioAsset(sentence), sentenceMediaFilename(sentence));
}

export function uploadWordAudioBatch(words) {
  return Promise.all(words.map(uploadWordAudio));
}

export function uploadSentenceAudioBatch(sentences) {
  return Promise.all(sentences.map(uploadSentenceAudio));
}
