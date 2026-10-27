import { onMounted, onUnmounted, shallowRef, watch } from 'vue';
import { cachedBeanLayout, cacheBeanLayout } from './layouts';
import type { BeanLayout } from './layouts';
export function useCoffeeLayout(ids: () => string[]) {
  const layout = shallowRef<BeanLayout | null>(null);
  let worker: Worker | undefined;
  let generation = 0;
  function stop() {
    worker?.terminate();
    worker = undefined;
  }
  function run() {
    stop();
    if (layout.value || document.hidden) return;
    const version = generation;
    const selected = ids();
    worker = new Worker(new URL('./layout.worker.ts', import.meta.url), { type: 'module' });
    worker.onmessage = (event: MessageEvent<BeanLayout>) => {
      if (version !== generation) return;
      cacheBeanLayout(selected, event.data);
      layout.value = event.data;
      stop();
    };
    worker.postMessage(selected);
  }
  function rebuild() {
    generation++;
    stop();
    layout.value = cachedBeanLayout(ids()) ?? null;
    run();
  }
  function visibility() {
    if (document.hidden) stop();
    else if (!layout.value) run();
  }
  watch(() => JSON.stringify(ids()), rebuild);
  onMounted(() => {
    document.addEventListener('visibilitychange', visibility);
    rebuild();
  });
  onUnmounted(() => {
    generation++;
    stop();
    document.removeEventListener('visibilitychange', visibility);
  });
  return { layout };
}
