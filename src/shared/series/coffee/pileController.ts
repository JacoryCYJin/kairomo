import { cachedPile, cachedPilePrefix, rememberPile } from './pileCache.ts';
import type { PileLayout } from './beanPile';
export interface PileRequest {
  seed: string;
  ids: string[];
}
interface Job {
  request: PileRequest;
  worker: Worker;
  timeout?: ReturnType<typeof setTimeout>;
}
/** Own computation independently of which period is currently visible, including during rest. */
export function pileController(
  createWorker: () => Worker,
  changed: () => void,
  failed: (seed: string) => void = () => {},
) {
  const jobs = new Map<string, Job>();
  let requests: PileRequest[] = [];
  let suspended = false;
  let disposed = false;
  function stop(seed: string) {
    const job = jobs.get(seed);
    jobs.delete(seed);
    clearTimeout(job?.timeout);
    job?.worker.terminate();
  }
  function same(a: PileRequest, b: PileRequest) {
    return (
      a.seed === b.seed && a.ids.length === b.ids.length && a.ids.every((id, i) => id === b.ids[i])
    );
  }
  function prepare(next: PileRequest[]) {
    if (disposed) return;
    requests = next.map((request) => ({ ...request, ids: [...request.ids] }));
    for (const [seed, job] of jobs) {
      if (!requests.some((request) => same(request, job.request))) stop(seed);
    }
    for (const request of requests) {
      if (cachedPile(request.ids, request.seed)) continue;
      if (!request.ids.length) {
        rememberPile({ seed: request.seed, poses: [], evaluatedIds: [], capped: false });
        continue;
      }
      if (suspended || jobs.has(request.seed)) continue;
      let worker: Worker;
      try {
        worker = createWorker();
      } catch {
        failed(request.seed);
        continue;
      }
      const job: Job = { request, worker };
      jobs.set(request.seed, job);
      worker.onmessage = (event: MessageEvent<PileLayout>) => {
        if (disposed || jobs.get(request.seed) !== job) return;
        const result = event.data;
        // A terminated/obsolete job must never publish another period's beans.
        if (
          result.seed === request.seed &&
          result.evaluatedIds.every((id, index) => request.ids[index] === id) &&
          (result.capped || result.evaluatedIds.length === request.ids.length)
        ) {
          rememberPile(result);
        }
        stop(request.seed);
        changed();
      };
      worker.onerror = () => {
        if (jobs.get(request.seed) !== job) return;
        stop(request.seed);
        failed(request.seed);
        changed();
      };
      job.timeout = setTimeout(() => {
        if (jobs.get(request.seed) !== job) return;
        stop(request.seed);
        failed(request.seed);
        changed();
      }, 10000);
      try {
        worker.postMessage(request);
      } catch {
        stop(request.seed);
        failed(request.seed);
      }
    }
    changed();
  }
  return {
    prepare,
    read(request: PileRequest) {
      const exact = cachedPile(request.ids, request.seed);
      return {
        poses: (exact ?? cachedPilePrefix(request.ids, request.seed))?.poses ?? [],
        calculating: !exact && request.ids.length > 0,
      };
    },
    suspend() {
      suspended = true;
      for (const seed of [...jobs.keys()]) stop(seed);
    },
    resume() {
      suspended = false;
      prepare(requests);
    },
    dispose() {
      disposed = true;
      for (const seed of [...jobs.keys()]) stop(seed);
    },
  };
}
