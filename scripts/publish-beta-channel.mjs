import { readFile } from 'node:fs/promises';
import { Buffer } from 'node:buffer';
import { createHash } from 'node:crypto';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
const root = fileURLToPath(new URL('../', import.meta.url));
const run = promisify(execFile);
const args = process.argv.slice(2);
if (args.length > 1 || (args.length === 1 && args[0] !== '--dry-run'))
  throw new Error('Usage: node scripts/publish-beta-channel.mjs [--dry-run]');
const dryRun = args.includes('--dry-run');
const repo = 'JacoryCYJin/kairomo';
const gh = async (...parameters) =>
  (await run('gh', parameters, { cwd: root, maxBuffer: 4 * 1024 * 1024 })).stdout;
const config = JSON.parse(await readFile(path.join(root, 'src-tauri/tauri.conf.json'), 'utf8'));
const version = config.version;
function betaParts(value) {
  const match = /^(\d+)\.(\d+)\.(\d+)-beta\.(\d+)$/.exec(value);
  if (!match) throw new Error('Invalid Beta version');
  return match.slice(1).map(BigInt);
}
const desired = betaParts(version);
const tag = `v${version}`;
const targetCommit = (await gh('api', `repos/${repo}/commits/${tag}`, '--jq', '.sha')).trim();
if (!/^[a-f0-9]{40}$/.test(targetCommit)) throw new Error('Could not resolve the version tag');
const directory = path.join(root, 'target/beta', version);
const manifestFile = path.join(directory, 'latest.json');
const manifestBytes = await readFile(manifestFile);
const manifest = JSON.parse(manifestBytes.toString());
if (manifest.version !== version || !manifest.platforms || !Object.keys(manifest.platforms).length)
  throw new Error('Manifest does not match configured version');
const release = JSON.parse(
  await gh('release', 'view', tag, '--repo', repo, '--json', 'assets,isDraft,isPrerelease'),
);
if (release.isDraft || !release.isPrerelease)
  throw new Error('Publish the versioned Beta prerelease before promoting the channel');
async function verifiedAsset(name) {
  const asset = release.assets.find((item) => item.name === name);
  const bytes = await readFile(path.join(directory, name));
  if (
    !asset ||
    asset.state !== 'uploaded' ||
    asset.digest !== `sha256:${createHash('sha256').update(bytes).digest('hex')}`
  )
    throw new Error(`Published asset does not match local artifact: ${name}`);
  return bytes;
}
await verifiedAsset('latest.json');
for (const platform of Object.values(manifest.platforms)) {
  const prefix = `https://github.com/${repo}/releases/download/${tag}/`;
  const name = platform.url?.startsWith(prefix) ? platform.url.slice(prefix.length) : '';
  if (!name || !/^[\w.-]+$/.test(name)) throw new Error('Invalid versioned update URL');
  await verifiedAsset(name);
  const signature = (await verifiedAsset(`${name}.sig`)).toString().trim();
  if (signature !== platform.signature) throw new Error('Manifest signature differs from release');
  const signed = Buffer.from(signature, 'base64').toString();
  if (
    !signed
      .split('\n')
      .some(
        (line) =>
          line.startsWith('trusted comment:') && line.split('\t').includes(`version:${version}`),
      )
  )
    throw new Error('Update signature is not bound to the release version');
}
let channel;
try {
  channel = JSON.parse(await gh('api', `repos/${repo}/releases/tags/beta-channel`));
} catch (error) {
  if (!String(error.stderr).includes('HTTP 404')) throw error;
}
if (channel) {
  if (channel.draft || !channel.prerelease)
    throw new Error('Beta channel must be a public prerelease');
  if (channel.assets.some((asset) => asset.name === 'latest.json')) {
    const current = JSON.parse(
      await gh(
        'release',
        'download',
        'beta-channel',
        '--repo',
        repo,
        '--pattern',
        'latest.json',
        '--output',
        '-',
      ),
    );
    const previous = betaParts(current.version);
    const firstDifference = desired.findIndex((part, index) => part !== previous[index]);
    if (firstDifference >= 0 && desired[firstDifference] < previous[firstDifference])
      throw new Error('Refusing to move the Beta channel to an older version');
    if (firstDifference === -1) {
      if (JSON.stringify(current) !== JSON.stringify(manifest))
        throw new Error('Same-version channel contents differ; publish a new Beta version');
      console.log(`Beta channel already points to ${version}.`);
      process.exit(0);
    }
  }
}
if (dryRun) {
  console.log(
    `Verified ${tag}; would ${channel ? 'update' : 'create'} beta-channel/latest.json. No remote changes.`,
  );
  process.exit(0);
}
if (!channel) {
  await gh(
    'release',
    'create',
    'beta-channel',
    '--repo',
    repo,
    '--target',
    targetCommit,
    '--prerelease',
    '--latest=false',
    '--title',
    'Kairomo Beta 更新通道',
    '--notes',
    '此发布仅提供固定的 Beta 更新清单。安装包和更新说明请查看对应版本的 Beta Release。',
    manifestFile,
  );
} else {
  await gh('release', 'upload', 'beta-channel', manifestFile, '--repo', repo, '--clobber');
}
const response = await globalThis.fetch(
  `https://github.com/${repo}/releases/download/beta-channel/latest.json`,
  {
    signal: globalThis.AbortSignal.timeout(30_000),
  },
);
if (!response.ok || !Buffer.from(await response.arrayBuffer()).equals(manifestBytes))
  throw new Error('Channel changed, but public HTTPS verification failed; inspect before retrying');
console.log(`Published and verified Beta channel: ${version}`);
