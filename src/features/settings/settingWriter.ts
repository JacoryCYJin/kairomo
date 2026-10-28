/** Serialize preference writes without dropping edits while another save is in flight. */
export function settingWriter<T>(apply: (value: T) => void, failed: () => void) {
  let pending = Promise.resolve();
  return (operation: () => Promise<T>): Promise<void> => {
    pending = pending.then(async () => {
      try {
        apply(await operation());
      } catch {
        failed();
      }
    });
    return pending;
  };
}
