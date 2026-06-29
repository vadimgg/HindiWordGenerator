import { readySentences } from './state.mjs';

function quote(value) {
  const text = String(value ?? '').trim();
  if (!text) return '""';
  if (/^[A-Za-z0-9_./:=@+-]+$/.test(text)) return text;
  return `"${text.replaceAll('"', '\\"')}"`;
}

function idsArg(ids) {
  const values = [...(ids ?? [])];
  return values.length ? values.map((id) => `--id ${quote(id)}`).join(' ') : '--all';
}

function sectionArg(section) {
  return section ? `--section ${quote(section)}` : '';
}

export function commandLibrary(state, ctx = {}) {
  const section = ctx.section || state?.sentences?.[0]?.section || 'Chapter 02';
  const collection = state?.workspace?.name || 'Complete Hindi';
  const limit = Number(ctx.limit || 20);
  const importPath = ctx.importPath || '<package-dir>';
  const deck = ctx.ankiDeck || state?.config?.anki?.deck || 'Hindi::Sentences';
  const packageDest = ctx.packageDest || state?.config?.package?.destination || 'packages/sentences';
  const packageFormat = ctx.packageFormat || state?.config?.package?.format || 'json';
  const audioBackend = ctx.audioBackend || state?.config?.audio?.backend || 'gtts';
  const audioVoice = ctx.audioVoice || state?.config?.audio?.voice || '';
  const selection = ctx.selectedIds?.size ? idsArg(ctx.selectedIds) : '--all';
  const readyCount = readySentences(state?.sentences ?? []).length;

  const audioVoicePart = audioVoice ? ` --voice ${quote(audioVoice)}` : '';
  return {
    'generate.extract': {
      title: 'Extract raw text',
      description: 'Write or print an extract prompt. The apply step validates and inserts draft sentences.',
      command: `lingo extract raw/source.md --collection ${quote(collection)} ${sectionArg(section)} --out runs/extract/${slug(section)}/prompt.md --watch`,
    },
    'generate.enrich': {
      title: 'Enrich bounded batch',
      description: 'Claim the next draft rows, emit a prompt, and prevent overlap with future prompts.',
      command: `lingo enrich --limit ${limit} --out runs/enrich/${slug(section)}/prompt.md --watch`,
    },
    'generate.reset': {
      title: 'Reset abandoned claims',
      description: 'Return stuck enriching rows to draft so they can be claimed again.',
      command: 'lingo enrich --reset',
    },
    import: {
      title: 'Import package',
      description: 'Merge a JSON package or db copy into the canonical library.',
      command: `lingo import --from ${quote(importPath)}`,
    },
    organize: {
      title: 'List / organize',
      description: 'Inspect the canonical sentence rows. Move, edit, tag, and delete commands are row-specific.',
      command: `lingo ls ${sectionArg(section)} --json`.replace(/\s+/g, ' ').trim(),
    },
    'organize.move': {
      title: 'Move selected row',
      description: 'Use after drag-reorder or when changing sections.',
      command: ctx.focusedId ? `lingo organize move ${quote(ctx.focusedId)} --to <ord>` : 'lingo organize move <sentence-id> --to <ord>',
    },
    'organize.edit': {
      title: 'Edit selected row',
      description: 'Editing from CLI/UI marks fields as human-authored.',
      command: ctx.focusedId ? `lingo organize set ${quote(ctx.focusedId)} --english ${quote('<translation>')}` : 'lingo organize set <sentence-id> --english "..."',
    },
    words: {
      title: 'Lexicon',
      description: 'Read the projected words/meanings/occurrences from the library.',
      command: `lingo words --min-count ${Number(ctx.minCount || 1)} --json`,
    },
    audio: {
      title: 'Generate missing audio',
      description: 'Audio is a service over the library, not a pipeline gate.',
      command: `lingo audio --backend ${audioBackend}${audioVoicePart}`,
    },
    'audio.force': {
      title: 'Regenerate selection',
      description: 'Force re-synthesis for selected rows.',
      command: `lingo audio ${selection} --backend ${audioBackend}${audioVoicePart} --force`,
    },
    anki: {
      title: 'Export Anki',
      description: `${readyCount} ready sentence${readyCount === 1 ? '' : 's'} are exportable by default.`,
      command: `lingo export ${selection} --deck ${quote(deck)} --dest exports/lingo-sentences.apkg`,
    },
    package: {
      title: 'Export package',
      description: 'Default output is JSON; db copy is available for consumers that prefer SQLite.',
      command: `lingo package ${selection} --as ${packageFormat} --dest ${quote(packageDest)}`,
    },
    status: {
      title: 'Status',
      description: 'Summarize canonical db state and next action.',
      command: 'lingo status --json',
    },
    settings: {
      title: 'Persist settings',
      description: 'Viewer settings map to config.toml keys.',
      command: 'lingo config set <key> <value>',
    },
  };
}

export function commandForPage(page, state, ctx = {}) {
  const commands = commandLibrary(state, ctx);
  switch (page) {
    case 'generate': return commands['generate.enrich'].command;
    case 'import': return commands.import.command;
    case 'organize': return commands.organize.command;
    case 'words': return commands.words.command;
    case 'audio': return commands.audio.command;
    case 'anki': return commands.anki.command;
    case 'package': return commands.package.command;
    default: return commands.status.command;
  }
}

export function commandCardsForPage(page, state, ctx = {}) {
  const commands = commandLibrary(state, ctx);
  const keys = {
    generate: ['generate.extract', 'generate.enrich', 'generate.reset'],
    import: ['import'],
    organize: ['organize', 'organize.move', 'organize.edit'],
    words: ['words'],
    audio: ['audio', 'audio.force'],
    anki: ['anki'],
    package: ['package'],
  }[page] || ['status'];
  return keys.map((key) => ({ key, ...commands[key] }));
}

export function slug(value) {
  return String(value ?? 'run')
    .normalize('NFKD')
    .replace(/[^A-Za-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
    .toLowerCase() || 'run';
}

export const privateForTests = { quote, idsArg, sectionArg };
