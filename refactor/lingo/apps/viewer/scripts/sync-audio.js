import { lstat, mkdir, readlink, rm, symlink } from 'node:fs/promises';
import { join, relative } from 'node:path';

const root = process.cwd();
const projectRoot = process.env.LINGO_WORKSPACE_ROOT || join(root, '..');
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

  const relativeSource = relative(join(root, 'public'), sourceDir);
  const existing = await pathInfo(targetDir);
  if (existing?.isSymbolicLink()) {
    const currentTarget = await readlink(targetDir);
    if (currentTarget === relativeSource) {
      console.log('[sync-audio] public/audio already points at project audio.');
      return;
    }
    await rm(targetDir, { recursive: true, force: true });
  }

  if (existing) {
    await rm(targetDir, { recursive: true, force: true });
  }

  await symlink(relativeSource, targetDir, 'dir');
  console.log(`[sync-audio] Linked public/audio -> ${relativeSource}`);
}

main().catch((error) => {
  console.error('[sync-audio] Failed to link audio assets.');
  console.error(error);
  process.exitCode = 1;
});
