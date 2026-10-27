import assert from 'node:assert/strict';
import test from 'node:test';
import { compactOverflow } from './compactOverflow.ts';

test('exterior composition preserves every record and never changes interior geometry', () => {
  for (const count of [0, 1, 10, 26, 40]) {
    const inside = {
      id: 'inside',
      x: 240,
      y: 100,
      angle: 0.3,
      depth: 4,
      size: 1,
      squash: 0.8,
      inside: true,
      layer: 'body',
    };
    const input = [
      inside,
      ...Array.from({ length: count }, (_, i) => ({
        ...inside,
        id: `outside-${i}`,
        inside: false,
        layer: 'front',
      })),
    ];
    const original = JSON.parse(JSON.stringify(input));
    const result = compactOverflow(input);
    assert.equal(result.poses.length, input.length);
    assert.equal(new Set(result.poses.map((p) => p.id)).size, input.length);
    assert.equal(
      result.poses.find((p) => p.inside),
      inside,
    );
    assert.deepEqual(input, original);
    assert.deepEqual(result, compactOverflow(input));
    for (const pose of result.poses)
      assert.ok(Number.isFinite(pose.x + pose.y + pose.angle + pose.size));
    if (count === 0) assert.deepEqual(result.shadows, []);
  }
});
