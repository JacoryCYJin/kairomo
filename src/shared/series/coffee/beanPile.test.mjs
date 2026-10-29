import assert from 'node:assert/strict';
import test from 'node:test';
import { Body } from 'cannon-es';
import { createPileJob, PILE_MAX_HEIGHT } from './beanPile.ts';
function supported(bodies) {
  const spheres = bodies.map(({ body }) =>
    body.shapes.map((shape, i) => ({
      center: body.pointToWorldFrame(body.shapeOffsets[i]),
      radius: shape.radius,
    })),
  );
  const grounded = new Set();
  for (let i = 0; i < spheres.length; i++)
    if (spheres[i].some((s) => s.center.y - s.radius < 0.8)) grounded.add(i);
  for (let pass = 0; pass < bodies.length; pass++) {
    const before = grounded.size;
    for (let i = 0; i < spheres.length; i++) {
      if (grounded.has(i)) continue;
      for (const j of grounded)
        if (
          spheres[i].some((a) =>
            spheres[j].some((b) => a.center.distanceTo(b.center) <= a.radius + b.radius + 0.8),
          )
        ) {
          grounded.add(i);
          break;
        }
    }
    if (grounded.size === before) break;
  }
  assert.equal(grounded.size, bodies.length, 'every bean must touch the floor or a grounded pile');
}
function settle(ids, seed = 'test-period') {
  const job = createPileJob(ids, seed);
  let result;
  for (let i = 0; i < 2500 && !result; i++) result = job.step(24);
  assert.ok(result, 'the physical pile must settle rather than publishing moving bodies');
  return { job, result };
}
for (const count of [0, 1, 6, 20, 80, 121, 400, 1440])
  test(`${count} records produce a supported pile below the ceiling`, () => {
    const ids = Array.from({ length: count }, (_, i) => `focus-${i}`);
    const { job, result } = settle(ids);
    assert.equal(ids.length, count, 'input records remain complete');
    assert.ok(result.poses.length <= count);
    if (count <= 6) assert.equal(result.poses.length, count);
    assert.deepEqual(result.evaluatedIds, ids.slice(0, result.evaluatedIds.length));
    assert.deepEqual(
      new Set(result.poses.map((p) => p.id)),
      new Set(ids.slice(0, result.poses.length)),
    );
    assert.equal(job.bodies.length, result.poses.length);
    if (result.capped) assert.equal(result.evaluatedIds.length, result.poses.length + 1);
    else assert.equal(result.poses.length, count);
    for (const p of result.poses) {
      assert.ok(Number.isFinite(p.x + p.y + p.depth + p.angle));
      assert.ok(p.x >= 10 && p.x <= 350);
      assert.ok(p.size >= 36 && p.size <= 44, 'nearby beans must stay large at high counts');
      assert.ok(p.squash >= 0.7 && p.squash <= 1);
    }
    for (const { body } of job.bodies) {
      assert.equal(body.sleepState, Body.SLEEPING);
      assert.ok(body.position.y >= 7, 'no bean falls through the floor');
    }
    supported(job.bodies);
    if (count >= 6) {
      const depths = result.poses.map((p) => p.depth);
      assert.ok(Math.max(...depths) - Math.min(...depths) > 5);
    }
    const height = Math.max(0, ...result.poses.map((p) => p.y + p.size * 0.44));
    assert.ok(height <= PILE_MAX_HEIGHT, 'the whole SVG stays below the button ceiling');
    if (count >= 121) assert.equal(result.capped, true);
    assert.equal(job.step(), result, 'settled poses stay still');
    job.dispose();
  });
test('geometry is deterministic, and cancelled work cannot publish a layout', () => {
  const a = settle(['one', 'two', 'three']);
  const b = settle(['one', 'two', 'three']);
  assert.deepEqual(a.result, b.result);
  a.job.dispose();
  b.job.dispose();
  const cancelled = createPileJob(['cancelled'], 'test-period');
  cancelled.dispose();
  assert.equal(cancelled.step(), undefined);
});

test('additional completions freeze the same complete supported prefix once capped', () => {
  const ids = Array.from({ length: 1440 }, (_, i) => 'unchanging-' + i);
  const capped = settle(ids.slice(0, 400));
  const more = settle(ids);
  assert.equal(capped.result.capped, true);
  assert.deepEqual(capped.result, more.result);
  const lastSafe = settle(ids.slice(0, capped.result.poses.length));
  assert.equal(lastSafe.result.capped, false);
  assert.deepEqual(lastSafe.result.poses, capped.result.poses);
  capped.job.dispose();
  more.job.dispose();
  lastSafe.job.dispose();
});

test('independent period seeds produce different physical layouts with identical records', () => {
  const ids = Array.from({ length: 40 }, (_, i) => 'same-' + i);
  const a = settle(ids, 'day-A'),
    b = settle(ids, 'day-B'),
    restored = settle(ids, 'day-A');
  assert.notDeepEqual(a.result.poses, b.result.poses);
  assert.deepEqual(a.result, restored.result);
  supported(a.job.bodies);
  supported(b.job.bodies);
  a.job.dispose();
  b.job.dispose();
  restored.job.dispose();
});
