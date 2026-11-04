import { readFileSync } from 'node:fs';
import { join } from 'node:path';

const root = join(import.meta.dirname, '..');
const readWindows = (file) => {
  const config = JSON.parse(readFileSync(join(root, 'src-tauri', file), 'utf8'));
  return config.app?.windows ?? [];
};

const devWindows = readWindows('tauri.conf.json');
const betaWindows = readWindows('tauri.beta.conf.json');
const labels = new Set([...devWindows, ...betaWindows].map((window) => window.label));
const ignored = new Set(['title']);
const errors = [];

for (const label of labels) {
  const dev = devWindows.find((window) => window.label === label);
  const beta = betaWindows.find((window) => window.label === label);
  if (!dev || !beta) {
    errors.push(`${label}: window exists in only one config`);
    continue;
  }
  const keys = new Set([...Object.keys(dev), ...Object.keys(beta)]);
  for (const key of keys) {
    if (ignored.has(key)) continue;
    if (JSON.stringify(dev[key]) !== JSON.stringify(beta[key])) {
      errors.push(
        `${label}.${key}: Dev=${JSON.stringify(dev[key])} Beta=${JSON.stringify(beta[key])}`,
      );
    }
  }
}

if (errors.length) {
  console.error('Window config mismatch between tauri.conf.json and tauri.beta.conf.json:');
  for (const error of errors) console.error(`- ${error}`);
  process.exit(1);
}

console.log('Window config: Dev and Beta window geometry/behavior match.');
