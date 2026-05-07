import { readdir, readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { getSentenceTokenIssue, hasExactSentenceTokens } from '../scripts/quality/sentenceTokens.js';

async function readJsonFiles(dir) {
  try {
    const files = (await readdir(dir)).filter(file => file.endsWith('.json')).sort();
    const loaded = await Promise.all(
      files.map(async (file) => {
        try {
          return {
            file,
            stem: file.replace('.json', ''),
            data: JSON.parse(await readFile(join(dir, file), 'utf-8')),
          };
        } catch (error) {
          console.warn(`[viewer] Skipping invalid JSON file: ${join(dir, file)}`);
          console.warn(error);
          return null;
        }
      })
    );
    return loaded.filter(Boolean);
  } catch (error) {
    console.warn(`[viewer] No readable JSON directory: ${dir}`);
    return [];
  }
}

function metadataFromBatch(data, fallback) {
  const title = typeof data === 'object' && !Array.isArray(data)
    ? data.title
    : undefined;
  const subtitle = typeof data === 'object' && !Array.isArray(data)
    ? data.subtitle
    : undefined;
  if (typeof title !== 'string' || !title.trim()) {
    console.warn(`[viewer] Missing title in generated batch: ${fallback}`);
  }
  const normalizedTitle = title || fallback.replace(/_/g, ' ');
  const normalizedSubtitle = typeof subtitle === 'string' ? subtitle : '';
  return {
    title: normalizedTitle,
    subtitle: normalizedSubtitle,
    label: [normalizedTitle, normalizedSubtitle].filter(Boolean).join(' '),
  };
}

function buildWordFiles(wordFiles) {
  return wordFiles.map(({ stem, data }) => {
    const audioBatch = stem;
    const meta = metadataFromBatch(data, stem);
    const rawWords = Array.isArray(data) ? data : (data.words ?? []);
    const words = rawWords.map(word => ({
      ...word,
      audioBatch,
      title: meta.title,
      subtitle: meta.subtitle,
      groupLabel: meta.label,
    }));
    return { title: meta.label, words };
  });
}

function buildWordGroups(allFiles) {
  let offset = 0;
  const merged = new Map();
  let groupIndex = 0;

  for (const file of allFiles.filter(item => item.words.length > 0)) {
    if (!merged.has(file.title)) {
      merged.set(file.title, { id: String(groupIndex++), title: file.title, words: [] });
    }
    const group = merged.get(file.title);
    for (const word of file.words) {
      group.words.push({ w: word, i: offset++ });
    }
  }

  return [...merged.values()];
}

function buildSentenceFiles(sentenceFiles) {
  return sentenceFiles
    .map(({ stem, data }) => {
      const audioBatch = stem;
      const meta = metadataFromBatch(data, stem);
      const rawSentences = (typeof data === 'object' && !Array.isArray(data) && data.sentences)
        ? data.sentences
        : Array.isArray(data) ? data : [];
      const sentences = rawSentences.map(sentence => ({
        ...sentence,
        audioBatch,
        title: meta.title,
        subtitle: meta.subtitle,
        groupLabel: meta.label,
      }));
      return { groupLabel: meta.label, stem, sentences };
    })
    .filter(batch => batch.sentences.length > 0);
}

function buildSentenceGroups(sentenceBatchFiles) {
  const merged = new Map();

  for (const batch of sentenceBatchFiles) {
    if (!merged.has(batch.groupLabel)) {
      merged.set(batch.groupLabel, { label: batch.groupLabel, sentences: [], batches: [] });
    }
    const group = merged.get(batch.groupLabel);
    group.batches.push(batch.stem);
    group.sentences.push(...batch.sentences);
  }

  return [...merged.values()];
}

function buildQaIssues(allSentences) {
  return allSentences.flatMap((sentence, index) => {
    const issues = [];
    const tokenIssue = getSentenceTokenIssue(sentence);
    if (tokenIssue) {
      issues.push({
        type: 'tokens',
        sentenceIndex: index,
        label: tokenIssue.reason,
        hindi: sentence.hindi ?? '',
        english: sentence.english ?? '',
        groupLabel: sentence.groupLabel ?? '',
        expected: tokenIssue.expected,
        actual: tokenIssue.actual,
      });
    }
    if (!sentence.audio) {
      issues.push({
        type: 'audio',
        sentenceIndex: index,
        label: 'missing audio path',
        hindi: sentence.hindi ?? '',
        english: sentence.english ?? '',
        groupLabel: sentence.groupLabel ?? '',
      });
    }
    return issues;
  });
}

export async function loadGeneratedData(projectRoot) {
  const wordFiles = await readJsonFiles(join(projectRoot, 'output', 'words'));
  const allFiles = buildWordFiles(wordFiles);
  const allWords = allFiles.flatMap(file => file.words);
  const wordGroups = buildWordGroups(allFiles);
  const wordGroupTitles = new Array(allWords.length).fill('Hindi Vocabulary');
  wordGroups.forEach(group => group.words.forEach(({ i }) => { wordGroupTitles[i] = group.title; }));

  const sentenceFiles = await readJsonFiles(join(projectRoot, 'output', 'sentences'));
  const sentenceBatchFiles = buildSentenceFiles(sentenceFiles);
  const sentenceGroups = buildSentenceGroups(sentenceBatchFiles);
  const allSentences = sentenceGroups.flatMap(group => group.sentences);
  const sentenceGroupLabels = sentenceGroups.flatMap(group =>
    group.sentences.map(() => group.label)
  );

  const wordSearchIndex = allWords.map((word, i) => ({
    i,
    h: word.hindi,
    r: word.romanisation,
    e: word.english,
    d: word.date_added ?? '',
  }));
  const sentenceSearchIndex = allSentences.map((sentence, i) => ({
    i,
    h: sentence.hindi ?? '',
    r: sentence.romanisation ?? '',
    e: sentence.english ?? '',
    d: sentence.date_added ?? '',
    group: sentenceGroupLabels[i] ?? '',
  }));
  const hoverData = allWords.map((word, i) => ({
    i,
    hindi: word.hindi,
    roman: word.romanisation,
    english: String(word.english ?? '').split(',')[0].trim(),
    forms: (word.forms || []).map(form => ({ h: form.hindi, r: form.roman })),
  }));

  const dataHealth = {
    wordFiles: wordFiles.length,
    sentenceFiles: sentenceFiles.length,
    totalWords: allWords.length,
    totalSentences: allSentences.length,
    wordAudioReady: allWords.filter(word => Boolean(word.audio)).length,
    sentenceAudioReady: allSentences.filter(sentence => Boolean(sentence.audio)).length,
    sentenceTokenReady: allSentences.filter(hasExactSentenceTokens).length,
  };

  return {
    wordFiles,
    sentenceFiles,
    allWords,
    wordGroups,
    wordGroupTitles,
    wordSearchIndex,
    sentenceSearchIndex,
    hoverData,
    allSentences,
    sentenceGroups,
    dataHealth,
    qaIssues: buildQaIssues(allSentences),
  };
}
