import assert from 'node:assert/strict';
import test from 'node:test';
import { pileController } from './pileController.ts';
import { cachedPilePrefix, rememberPile } from './pileCache.ts';
function layout(seed, ids, capped = false) {
  return {
    seed,
    evaluatedIds: [...ids],
    capped,
    poses: ids.map((id) => ({
      id,
      x: 20,
      y: 20,
      size: 42,
      angle: 0,
      depth: 0,
      squash: 1,
      shadowY: 0,
    })),
  };
}
function harness() {
  const workers = [];
  let updates = 0;
  const controller = pileController(
    () => {
      const worker = {
        terminated: false,
        postMessage(request) {
          this.request = request;
        },
        terminate() {
          this.terminated = true;
        },
        deliver(result = layout(this.request.seed, this.request.ids)) {
          this.onmessage({ data: result });
        },
      };
      workers.push(worker);
      return worker;
    },
    () => updates++,
  );
  return { controller, workers, updates: () => updates };
}
test('appending a completion keeps the supported old pile until the replacement is ready', () => {
  const seed = 'append';
  const old = layout(seed, ['a', 'b']);
  rememberPile(old);
  const { controller, workers } = harness();
  const request = { seed, ids: ['a', 'b', 'c'] };
  controller.prepare([request]);
  assert.equal(controller.read(request).poses, old.poses);
  assert.equal(controller.read(request).calculating, true);
  workers[0].deliver();
  assert.equal(controller.read(request).poses.length, 3);
  assert.equal(controller.read(request).calculating, false);
  controller.dispose();
});
test('both periods prepare once; repeated reconciliation and rest do not restart work', () => {
  const { controller, workers } = harness();
  const requests = [
    { seed: 'day-ready', ids: ['a'] },
    { seed: 'week-ready', ids: ['a', 'b'] },
  ];
  controller.prepare(requests);
  controller.resume();
  controller.prepare(requests);
  assert.equal(workers.length, 2);
  assert.ok(workers.every((worker) => !worker.terminated));
  workers.forEach((worker) => worker.deliver());
  for (const request of requests) assert.equal(controller.read(request).calculating, false);
  controller.prepare(requests);
  assert.equal(workers.length, 2);
  controller.dispose();
});
test('a hidden window cancels work and reopening resumes, rejecting the old worker', () => {
  const { controller, workers } = harness();
  const request = { seed: 'resume', ids: ['a'] };
  controller.prepare([request]);
  controller.suspend();
  assert.equal(workers[0].terminated, true);
  controller.prepare([request]);
  assert.equal(workers.length, 1);
  controller.resume();
  assert.equal(workers.length, 2);
  workers[0].deliver();
  assert.equal(controller.read(request).poses.length, 0);
  workers[1].deliver();
  assert.equal(controller.read(request).poses.length, 1);
  controller.dispose();
});
test('rapid record changes and period rollover cannot publish obsolete results', () => {
  const { controller, workers } = harness();
  const first = { seed: 'rapid', ids: ['a'] };
  const second = { seed: 'rapid', ids: ['a', 'b'] };
  controller.prepare([first]);
  controller.prepare([second]);
  workers[0].deliver();
  assert.equal(controller.read(second).poses.length, 0);
  workers[1].deliver();
  const next = { seed: 'next-day', ids: ['a', 'b'] };
  controller.prepare([next]);
  assert.equal(controller.read(next).poses.length, 0);
  controller.dispose();
  workers[2].deliver();
  assert.equal(controller.read(next).poses.length, 0);
});
test('deleted or reordered records never reuse unsupported geometry', () => {
  rememberPile(layout('prefix-only', ['a', 'b', 'c']));
  assert.equal(cachedPilePrefix(['b', 'c'], 'prefix-only'), undefined);
  assert.equal(cachedPilePrefix(['a', 'c', 'b'], 'prefix-only'), undefined);
  assert.equal(cachedPilePrefix(['a', 'b', 'c'], 'other-period'), undefined);
});
test('capped piles need no new worker and empty periods contain no beans', () => {
  rememberPile(layout('capped-controller', ['a', 'b'], true));
  const { controller, workers } = harness();
  const capped = { seed: 'capped-controller', ids: ['a', 'b', 'c'] };
  const empty = { seed: 'empty-controller', ids: [] };
  controller.prepare([capped, empty]);
  assert.equal(workers.length, 0);
  assert.equal(controller.read(capped).poses.length, 2);
  assert.deepEqual(controller.read(empty), { poses: [], calculating: false });
  controller.dispose();
});
test('worker failure can recover on reopen', () => {
  const { controller, workers } = harness();
  const request = { seed: 'retry-controller', ids: ['a'] };
  controller.prepare([request]);
  workers[0].onerror();
  controller.resume();
  assert.equal(workers.length, 2);
  workers[1].deliver();
  assert.equal(controller.read(request).calculating, false);
  controller.dispose();
});

test('worker startup and message failures report an error instead of trapping startup', () => {
  const request = { seed: 'failed-startup-controller', ids: ['a'] };
  const errors = [];
  const broken = pileController(
    () => {
      throw new Error('worker unavailable');
    },
    () => {},
    (seed) => errors.push(seed),
  );
  broken.prepare([request]);
  assert.deepEqual(errors, [request.seed]);
  broken.dispose();
  let terminated = false;
  const postFailure = pileController(
    () => ({
      postMessage() {
        throw new Error('unavailable');
      },
      terminate() {
        terminated = true;
      },
    }),
    () => {},
    (seed) => errors.push(seed),
  );
  postFailure.prepare([request]);
  assert.equal(terminated, true);
  assert.equal(errors.length, 2);
  postFailure.dispose();
});
