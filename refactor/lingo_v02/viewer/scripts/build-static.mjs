import { mkdir, rm, copyFile, readdir, stat } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const dist = join(root, 'dist');

async function copyDir(from, to) {
  await mkdir(to, { recursive: true });
  for (const entry of await readdir(from)) {
    const source = join(from, entry);
    const target = join(to, entry);
    const info = await stat(source);
    if (info.isDirectory()) await copyDir(source, target);
    else await copyFile(source, target);
  }
}

await rm(dist, { recursive: true, force: true });
await mkdir(dist, { recursive: true });
await copyFile(join(root, 'src/pages/index.astro'), join(dist, 'index.html'));
await copyDir(join(root, 'public/viewer'), join(dist, 'viewer'));
console.log(`built static viewer -> ${dist}`);
