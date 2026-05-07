/**
 * Audio asset contract shared by browser playback and Anki media export.
 */

const AUDIO_PREFIX = 'audio/';
const MP3_EXT = '.mp3';

function cleanAudioPath(audio) {
  if (typeof audio !== 'string') return undefined;
  const trimmed = audio.trim();
  if (!trimmed) return undefined;
  if (/^[a-z][a-z0-9+.-]*:/i.test(trimmed) || trimmed.startsWith('//')) return undefined;

  const relativePath = trimmed.startsWith('/') ? trimmed.slice(1) : trimmed;
  if (!relativePath.startsWith(AUDIO_PREFIX)) return undefined;
  if (!relativePath.endsWith(MP3_EXT)) return undefined;
  if (relativePath.split('/').some(part => part === '..' || part === '')) return undefined;

  return relativePath;
}

export function audioMediaFilenameFromPath(relativePath) {
  const withoutPrefix = relativePath.startsWith(AUDIO_PREFIX)
    ? relativePath.slice(AUDIO_PREFIX.length)
    : relativePath;
  return withoutPrefix.replaceAll('/', '__');
}

export function resolveAudioAsset(audio) {
  const path = cleanAudioPath(audio);
  if (!path) return undefined;
  return {
    path,
    browserSrc: `/${path}`,
    mediaFilename: audioMediaFilenameFromPath(path),
  };
}

export function resolveWordAudioAsset(word) {
  return resolveAudioAsset(word?.audio);
}

export function resolveSentenceAudioAsset(sentence) {
  return resolveAudioAsset(sentence?.audio);
}

export function isValidAudioPath(audio) {
  return Boolean(cleanAudioPath(audio));
}
