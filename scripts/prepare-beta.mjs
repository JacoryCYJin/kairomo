import { readFile, mkdir, copyFile, writeFile } from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { arch } from 'node:os';
const root = fileURLToPath(new URL('../', import.meta.url));
const config = JSON.parse(await readFile(path.join(root, 'src-tauri/tauri.conf.json'), 'utf8'));
const version = config.version;
const architecture = arch() === 'arm64' ? 'aarch64' : arch() === 'x64' ? 'x86_64' : null;
if (!architecture || !/^\d+\.\d+\.\d+-beta\.\d+$/.test(version))
  throw new Error('Unsupported Beta version or architecture');
const bundle = path.join(root, 'target/release/bundle');
const output = path.join(root, 'target/beta', version);
await mkdir(output, { recursive: true });
const prefix = `Kairomo_Beta_${version}_${architecture}`;
const archive = `${prefix}.app.tar.gz`;
const dmg = `${prefix}.dmg`;
const source = path.join(bundle, 'macos/Kairomo Beta.app.tar.gz');
await copyFile(source, path.join(output, archive));
await copyFile(`${source}.sig`, path.join(output, `${archive}.sig`));
await copyFile(
  path.join(bundle, 'dmg', `Kairomo Beta_${version}_${architecture}.dmg`),
  path.join(output, dmg),
);
const signature = (await readFile(`${source}.sig`, 'utf8')).trim();
const manifest = {
  version,
  notes:
    '建立主题套件基础设施，统一小窗、记录展示、记录标识、菜单栏与专属设置入口；保留咖啡现有视觉与数据，整理主题设置分类，精简版本更新区并收紧记录页底部留白。',
  pub_date: new Date().toISOString(),
  platforms: {
    [`darwin-${architecture}`]: {
      signature,
      url: `https://github.com/JacoryCYJin/kairomo/releases/download/v${version}/${archive}`,
    },
  },
};
await writeFile(path.join(output, 'latest.json'), JSON.stringify(manifest, null, 2) + '\n');
const checksums = [];
for (const name of [archive, `${archive}.sig`, dmg, 'latest.json']) {
  checksums.push(
    `${createHash('sha256')
      .update(await readFile(path.join(output, name)))
      .digest('hex')}  ${name}`,
  );
}
await writeFile(path.join(output, 'SHA256SUMS.txt'), checksums.join('\n') + '\n');
console.log(`Local Beta artifacts: ${output}`);
