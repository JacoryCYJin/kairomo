import { readdir, readFile } from 'node:fs/promises';
import path from 'node:path';
const root = new URL('../', import.meta.url).pathname;
async function files(dir) {
  const out = [];
  for (const entry of await readdir(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) out.push(...(await files(p)));
    else if (/\.(vue|ts)$/.test(p)) out.push(p);
  }
  return out;
}
const errors = [];
for (const file of await files(path.join(root, 'src'))) {
  const text = await readFile(file, 'utf8');
  const relative = path.relative(root, file);
  if (
    !relative.startsWith('src/shared/ipc/') &&
    /(?:from\s*|import\s*\()['"]@tauri-apps\//.test(text)
  )
    errors.push(`${relative}: desktop SDK must be accessed through shared/ipc`);
  const own = relative.match(/^src\/features\/([^/]+)/)?.[1];
  for (const match of text.matchAll(/(?:from\s*|import\s*\()['"]([^'"]+)['"]/g)) {
    const target = path.relative(
      path.join(root, 'src'),
      path.resolve(path.dirname(file), match[1]),
    );
    if (relative.startsWith('src/shared/') && /^(app|features)\//.test(target))
      errors.push(`${relative}: shared cannot depend on app/features`);
    if (own && target.startsWith('features/') && target.split('/')[1] !== own)
      errors.push(`${relative}: features cannot import another feature's internals`);
  }
}
const core = await readFile(path.join(root, 'crates/timer-core/Cargo.toml'), 'utf8');
if (/tauri|rusqlite|timer-storage/.test(core))
  errors.push('timer-core must remain independent of desktop and database adapters');
const storage = await readFile(path.join(root, 'crates/timer-storage/Cargo.toml'), 'utf8');
if (/tauri|kairomo-desktop/.test(storage))
  errors.push('timer-storage must not depend on the desktop shell');
if (errors.length) {
  console.error(errors.join('\n'));
  process.exit(1);
}
console.log('Architecture boundaries: passed.');
