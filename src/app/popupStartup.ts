/** Release native startup only after the latest complete state has flushed to the DOM. */
export function popupStartup(
  settle: () => Promise<void>,
  reveal: () => Promise<void>,
  completed: () => void,
  failed: () => void,
) {
  let revision = 0;
  let released = false;
  let disposed = false;
  return {
    async update(ready: boolean) {
      const current = ++revision;
      if (!ready || released || disposed) return;
      try {
        await settle();
        if (current !== revision || disposed || released) return;
        released = true;
        await reveal();
        if (!disposed) completed();
      } catch {
        released = false;
        if (!disposed) failed();
      }
    },
    dispose() {
      disposed = true;
      revision++;
    },
  };
}
