/**
 * Shared Anki deck-name helpers.
 */

export function composeDeckName(main, sub, fallbackMain = 'Hindi') {
  const cleanMain = main?.trim() || fallbackMain;
  const cleanSub = sub?.trim() || '';
  return cleanSub ? `${cleanMain}::${cleanSub}` : cleanMain;
}

export function quickExportDeckName(type, title) {
  const source = (title || 'Topic').replace(/\s+/g, ' ').trim();
  return `Hindi::${source}::${type === 'words' ? 'Words' : 'Sentences'}`;
}
