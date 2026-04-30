import { lstat, mkdir, rm, symlink } from 'node:fs/promises';
import { join, relative } from 'node:path';

const root = process.cwd();
const projectRoot = join(root, '..');
const sourceDir = join(projectRoot, 'audio');
const targetDir = join(root, 'public', 'audio');

async function pathInfo(path) {
  try {
    return await lstat(path);
  } catch {
    return null;
  }
}

async function main() {
  if (!(await pathInfo(sourceDir))) {
    console.warn(`[sync-audio] Skipping: ${sourceDir} not found.`);
    return;
  }

  await mkdir(join(root, 'public'), { recursive: true });

  const existing = await pathInfo(targetDir);
  if (existing?.isSymbolicLink()) {
    console.log('[sync-audio] public/audio already points at project audio.');
    return;
  }

  if (existing) {
    await rm(targetDir, { recursive: true, force: true });
  }

  const relativeSource = relative(join(root, 'public'), sourceDir);
  await symlink(relativeSource, targetDir, 'dir');
  console.log(`[sync-audio] Linked public/audio -> ${relativeSource}`);
}

main().catch((error) => {
  console.error('[sync-audio] Failed to link audio assets.');
  console.error(error);
  process.exitCode = 1;
});
