import { createBeanLayoutJob } from './layouts';

globalThis.onmessage = (event: MessageEvent<string[]>) => {
  const job = createBeanLayoutJob(event.data);
  let result = job.result;
  while (!result) result = job.step(240);
  globalThis.postMessage(result);
  job.dispose();
};
