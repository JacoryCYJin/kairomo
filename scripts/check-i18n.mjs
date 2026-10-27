import assert from 'node:assert/strict';
import fs from 'node:fs';
import { createRequire } from 'node:module';
import ts from 'typescript';
const source = fs.readFileSync(new URL('../src/shared/i18n.ts', import.meta.url), 'utf8');
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
}).outputText;
const translations = {};
new Function('require', 'exports', compiled)(createRequire(import.meta.url), translations);
const { locale, copy, formatBeanCount, localDate } = translations;
let expectedKeys;
for (const language of ['zh-CN', 'en-US', 'ja-JP']) {
  locale.value = language;
  const keys = Object.keys(copy.value).sort();
  if (expectedKeys) assert.deepEqual(keys, expectedKeys, `${language}: missing translations`);
  else expectedKeys = keys;
  for (const [key, text] of Object.entries(copy.value)) {
    assert.ok(typeof text === 'string' && text.trim(), `${language}: empty ${key}`);
  }
}
locale.value = 'en-US';
assert.equal(formatBeanCount(1), '1 coffee bean');
assert.equal(formatBeanCount(2), '2 coffee beans');
assert.equal(formatBeanCount(0), '0 coffee beans');
locale.value = 'ja-JP';
assert.equal(formatBeanCount(2), 'コーヒー豆2粒');
assert.match(localDate(new Date(2026, 9, 28, 12).getTime()), /10月28日/);
console.log(
  `Localization: ${expectedKeys.length} entries in each of 3 languages; counts and dates passed.`,
);
