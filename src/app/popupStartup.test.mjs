import assert from 'node:assert/strict';
import test from 'node:test';
import { setTimeout } from 'node:timers';
import { popupStartup } from './popupStartup.ts';
import { pileController } from '../shared/series/coffee/pileController.ts';
import { createPileJob } from '../shared/series/coffee/beanPile.ts';
function deferred() {
  let resolve;
  const promise = new Promise((done) => {
    resolve = done;
  });
  return { promise, resolve };
}
test('cold startup waits for a real physical pile, dimensions and DOM before revealing once', async () => {
  const frame = deferred();
  const order = [];
  let startup;
  const request = {
    seed: 'startup-cold-real-worker',
    ids: Array.from({ length: 17 }, (_, i) => 'cold-' + i),
  };
  const controller = pileController(
    () => {
      const worker = {
        terminate() {},
        postMessage(r) {
          const job = createPileJob(r.ids, r.seed);
          const advance = () => {
            const result = job.step();
            if (result) {
              job.dispose();
              this.onmessage({ data: result });
            } else setTimeout(advance, 0);
          };
          setTimeout(advance, 0);
        },
      };
      return worker;
    },
    () => {
      void startup.update(!controller.read(request).calculating);
    },
  );
  startup = popupStartup(
    async () => {
      order.push('settle');
      await frame.promise;
    },
    async () => {
      assert.equal(controller.read(request).poses.length, 17);
      order.push('show');
    },
    () => order.push('complete'),
    () => assert.fail('unexpected failure'),
  );
  controller.prepare([request]);
  assert.deepEqual(order, []);
  assert.equal(controller.read(request).poses.length, 0);
  await new Promise((resolve, reject) => {
    const started = Date.now();
    const inspect = () => {
      if (!controller.read(request).calculating) resolve();
      else if (Date.now() - started > 5000) reject(new Error('pile did not settle'));
      else setTimeout(inspect, 5);
    };
    inspect();
  });
  assert.deepEqual(order, ['settle']);
  frame.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.deepEqual(order, ['settle', 'show', 'complete']);
  await startup.update(true);
  assert.deepEqual(order, ['settle', 'show', 'complete']);
  controller.dispose();
  startup.dispose();
});
test('state changes while awaiting resize invalidate an obsolete initial frame', async () => {
  const frame = deferred();
  let shown = 0;
  const startup = popupStartup(
    () => frame.promise,
    async () => {
      shown++;
    },
    () => {},
    () => {},
  );
  const stale = startup.update(true);
  await startup.update(false);
  frame.resolve();
  await stale;
  assert.equal(shown, 0);
  await startup.update(true);
  assert.equal(shown, 1);
});
test('empty or rest screens can release without a bean worker; disposing prevents a late reveal', async () => {
  let shown = 0;
  const startup = popupStartup(
    async () => {},
    async () => {
      shown++;
    },
    () => {},
    () => {},
  );
  await startup.update(true);
  assert.equal(shown, 1);
  const frame = deferred();
  const disposed = popupStartup(
    () => frame.promise,
    async () => {
      shown++;
    },
    () => {},
    () => {},
  );
  const pending = disposed.update(true);
  disposed.dispose();
  frame.resolve();
  await pending;
  assert.equal(shown, 1);
});
test('a native readiness error is reported and can retry', async () => {
  let errors = 0,
    attempts = 0;
  const startup = popupStartup(
    async () => {},
    async () => {
      if (++attempts === 1) throw Error('IPC');
    },
    () => {},
    () => {
      errors++;
    },
  );
  await startup.update(true);
  assert.equal(errors, 1);
  await startup.update(true);
  assert.equal(attempts, 2);
});
