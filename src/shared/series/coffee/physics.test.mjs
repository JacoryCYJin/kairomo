import assert from 'node:assert/strict';
import test from 'node:test';
import { Body } from 'cannon-es';
import { createBeanWorld } from './physics.ts';
import { createBeanLayoutJob } from './layouts.ts';
function settle(count, prefix = 'preview') {
  const ids = Array.from({ length: count }, (_, i) => `${prefix}-${i}`);
  const world = createBeanWorld(ids);
  let steps = 0;
  while (!world.step() && steps++ < 7000) {
    // Advance the same fixed-step world used by the background layout worker.
  }
  assert.ok(steps < 7000, 'the solver must settle and stop');
  return world;
}
const inside = ({ body }) => (body.position.x / 70) ** 2 + (body.position.z / 36) ** 2 < 1.04;
for (const count of [0, 8, 20, 40, 80, 104, 120, 160]) {
  test(`${count} records settle with bounded display count and retain spatial depth`, () => {
    const world = settle(count);
    assert.equal(world.bodies.length, Math.min(count, 120));
    for (const { body } of world.bodies) {
      assert.equal(body.sleepState, Body.SLEEPING);
      assert.ok(Number.isFinite(body.position.x + body.position.y + body.position.z));
      assert.ok(body.position.y > 6, 'beans cannot fall through the tabletop');
    }
    if (count <= 40) assert.ok(world.bodies.every(inside), 'an unfilled jar must retain its beans');
    if (count >= 104) {
      assert.ok(
        world.bodies.some((b) => !inside(b)),
        'a full jar must naturally spill',
      );
      const interior = world.bodies.filter(inside);
      assert.ok(interior.length > 60, 'the jar must fill before spilling');
      assert.ok(Math.max(...interior.map((b) => b.body.position.y)) > 170);
    }
    if (count >= 8) {
      const depths = world.bodies.map((b) => b.body.position.z);
      assert.ok(Math.max(...depths) - Math.min(...depths) > 10, 'beans must occupy front and back');
    }
    const before = world.bodies.map((b) => b.body.position.clone());
    world.step();
    assert.deepEqual(
      world.bodies.map((b) => b.body.position),
      before,
      'settled positions must stay static',
    );
    world.dispose();
  });
}
test('every bean enters above the open mouth, never from preselected exterior piles', () => {
  const world = createBeanWorld(Array.from({ length: 104 }, (_, i) => `entry-${i}`));
  let previous = 0;
  for (let step = 0; step < 7000; step++) {
    const done = world.step();
    if (world.released > previous) {
      const body = world.bodies[previous].body;
      assert.ok(Math.abs(body.position.x) < 40 && Math.abs(body.position.z) < 20);
      assert.ok(body.position.y > 240);
      previous = world.released;
    }
    if (done) break;
  }
  assert.equal(previous, 104);
  world.dispose();
});
test('task identities do not leak from a partially filled curved vessel', () => {
  for (let seed = 0; seed < 3; seed++) {
    const world = settle(40, `curved-${seed}`);
    assert.ok(world.bodies.every(inside));
    world.dispose();
  }
});
test('projection preserves record identities, depth, caching and cancellation', () => {
  const ids = Array.from({ length: 104 }, (_, i) => `projection-${i}`);
  const job = createBeanLayoutJob(ids);
  let result;
  for (let i = 0; i < 10000 && !result; i++) result = job.step(240);
  assert.ok(result);
  assert.deepEqual(
    result.poses.map((p) => p.id),
    ids,
  );
  assert.equal(new Set(result.poses.map((p) => p.id)).size, 104);
  assert.ok(result.poses.some((p) => p.layer === 'front'));
  assert.ok(result.poses.some((p) => p.inside && p.depth < 0));
  assert.ok(result.poses.some((p) => p.inside && p.depth > 0));
  assert.ok(result.poses.every((p) => p.squash >= 0.68 && p.squash <= 1));
  assert.equal(createBeanLayoutJob(ids).result, result);
  const cancelled = createBeanLayoutJob(['cancelled']);
  cancelled.dispose();
  assert.equal(cancelled.step(), undefined);
  job.dispose();
});
