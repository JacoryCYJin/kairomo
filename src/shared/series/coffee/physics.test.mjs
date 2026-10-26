import assert from 'node:assert/strict';
import test from 'node:test';
import { createBeanWorld, SCENE_WIDTH, SCENE_HEIGHT } from './physics.ts';
import { createBeanLayoutJob } from './layouts.ts';
function settled(count, prefix = 'preview') {
  const world = createBeanWorld(Array.from({ length: count }, (_, i) => `${prefix}-${i}`));
  let steps = 0;
  while (!world.step() && steps < 2400) steps++;
  assert.ok(steps < 2400, 'solver must finish and release its frame loop');
  return world;
}
for (const n of [0, 8, 20, 40, 80, 110, 120, 500])
  test(`${n} records settle within a fixed non-scrolling scene`, () => {
    const world = settled(n);
    assert.equal(world.bodies.length, Math.min(n, 120));
    for (const b of world.bodies) {
      assert.ok(b.isSleeping);
      assert.ok(b.bounds.min.x >= 0 && b.bounds.max.x <= SCENE_WIDTH);
      assert.ok(b.bounds.min.y >= 0 && b.bounds.max.y <= SCENE_HEIGHT);
    }
    const outside = world.bodies.filter((b) => b.position.x < 168 || b.position.x > 312);
    if (n > 80) assert.ok(outside.length > 0, 'overflow must reach the exterior ground');
    if (n <= 40) assert.equal(outside.length, 0);
    const before = world.bodies.map((b) => ({ ...b.position }));
    world.step();
    assert.deepEqual(
      world.bodies.map((b) => ({ ...b.position })),
      before,
    );
    world.dispose();
  });
test('static layout publishes only after settling and reuses cached geometry', () => {
  const ids = Array.from({ length: 40 }, (_, i) => `static-${i}`);
  const job = createBeanLayoutJob(ids);
  assert.equal(job.result, undefined);
  assert.equal(job.step(1), undefined);
  let result;
  for (let i = 0; i < 100 && !result; i++) result = job.step();
  assert.ok(result);
  assert.equal(result.poses.length, 40);
  const next = createBeanLayoutJob(ids);
  assert.equal(next.result, result);
  job.dispose();
  next.dispose();
});
test('display saturation preserves the same first 120 records and cancellation stops computation', () => {
  const ids = Array.from({ length: 160 }, (_, i) => `cap-${i}`);
  const job = createBeanLayoutJob(ids);
  let result;
  for (let i = 0; i < 100 && !result; i++) result = job.step();
  assert.equal(result.poses.length, 120);
  assert.equal(createBeanLayoutJob(ids.slice(0, 120)).result, result);
  assert.equal(ids.length, 160);
  const cancelled = createBeanLayoutJob(['cancelled']);
  cancelled.dispose();
  assert.equal(cancelled.step(), undefined);
  job.dispose();
});
test('different task identities cannot tunnel through the exterior floor', () => {
  for (let seed = 0; seed < 10; seed++) {
    const world = settled(120, `fixture-${seed}`);
    assert.ok(world.bodies.every((b) => b.bounds.max.y <= SCENE_HEIGHT));
    assert.ok(world.bodies.every((b) => b.bounds.min.x >= 0 && b.bounds.max.x <= SCENE_WIDTH));
    world.dispose();
  }
});
test('overflow spreads into separate low piles on both sides of the open jar', () => {
  for (const count of [110, 120]) {
    const world = settled(count);
    for (const side of ['left', 'right']) {
      const pile = world.bodies.filter((b) =>
        side === 'left' ? b.position.x < 168 : b.position.x > 312,
      );
      assert.ok(pile.length > 5, 'both sides need a distinct pile');
      const width =
        Math.max(...pile.map((b) => b.bounds.max.x)) - Math.min(...pile.map((b) => b.bounds.min.x));
      const height =
        Math.max(...pile.map((b) => b.bounds.max.y)) - Math.min(...pile.map((b) => b.bounds.min.y));
      assert.ok(width > height, 'overflow should spread out rather than form a narrow column');
      assert.ok(height < 120, 'exterior piles must remain lower than the jar');
    }
    world.dispose();
  }
});
test('exterior beans appear only after the jar is filled across the entire mouth', () => {
  for (const count of [90, 100, 110, 120]) {
    const world = settled(count);
    const exterior = world.bodies.filter(
      (b) => world.foregroundDepth.has(b.label) || b.position.x < 168 || b.position.x > 312,
    );
    const interior = world.bodies.filter(
      (b) => !world.foregroundDepth.has(b.label) && b.position.x >= 176 && b.position.x <= 304,
    );
    if (exterior.length) {
      assert.ok(interior.length > 80, 'the initial estimate must not cap an unfilled jar');
      for (const x of [188, 208, 228, 248, 268, 288]) {
        const top = Math.min(
          ...interior
            .filter((b) => b.bounds.min.x <= x + 5 && b.bounds.max.x >= x - 5)
            .map((b) => b.bounds.min.y),
        );
        assert.ok(top <= 40, 'no empty band may remain below the rim while beans spill outside');
      }
    }
    world.dispose();
  }
});
test('foreground depth preserves one visible bean per record and appears only after filling', () => {
  for (const count of [20, 80, 110, 120, 160]) {
    const ids = Array.from({ length: count }, (_, i) => `preview-${i}`);
    const job = createBeanLayoutJob(ids);
    let result = job.result;
    for (let step = 0; step < 100 && !result; step++) result = job.step();
    assert.ok(result);
    assert.equal(result.poses.length, Math.min(count, 120));
    assert.equal(new Set(result.poses.map((pose) => pose.id)).size, result.poses.length);
    assert.deepEqual(
      result.poses.map((pose) => pose.id),
      ids.slice(0, 120),
    );
    const front = result.poses.filter((pose) => pose.layer === 'front');
    if (count <= 80) assert.equal(front.length, 0, 'unfilled jars cannot have foreground spills');
    else {
      assert.ok(front.length >= 3, 'overflow needs a visible foreground group');
      assert.ok(
        front.some((pose) => pose.x > 180 && pose.x < 300 && pose.y > 230),
        'some beans must lie in front of the jar, below its base',
      );
      assert.ok(
        Math.max(...front.map((pose) => pose.y)) - Math.min(...front.map((pose) => pose.y)) > 3,
        'foreground beans must not all sit on one horizontal line',
      );
    }
    assert.ok(
      result.poses.every(
        (pose) => pose.x > 0 && pose.x < SCENE_WIDTH && pose.y > 0 && pose.y + 11 < SCENE_HEIGHT,
      ),
    );
    job.dispose();
  }
});
