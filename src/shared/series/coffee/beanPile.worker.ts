import { createPileJob } from './beanPile';
globalThis.onmessage = (event: MessageEvent<{ ids: string[]; seed: string }>) => {
  const job = createPileJob(event.data.ids, event.data.seed);
  function advance() {
    const result = job.step();
    if (result) {
      globalThis.postMessage(result);
      job.dispose();
    } else setTimeout(advance, 0);
  }
  advance();
};
