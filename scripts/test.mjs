import { readdir } from 'node:fs/promises';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
async function collect(directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const nested = await Promise.all(
    entries.map((entry) => {
      const target = path.join(directory, entry.name);
      return entry.isDirectory()
        ? collect(target)
        : entry.name.endsWith('.test.mjs')
          ? [target]
          : [];
    }),
  );
  return nested.flat();
}
const result = spawnSync(
  process.execPath,
  ['--experimental-strip-types', '--test', ...(await collect(path.join(root, 'src'))).sort()],
  { cwd: root, stdio: 'inherit' },
);
if (result.error) throw result.error;
process.exit(result.status ?? 1);
