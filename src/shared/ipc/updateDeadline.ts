// Discovery has a 20s timeout and the manifest check a 30s timeout.
// Also bound the IPC wait if the native command never returns a response.
export async function withUpdateDeadline<T>(
  request: () => Promise<T>,
  milliseconds = 60_000,
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      Promise.resolve().then(request),
      new Promise<never>((_, reject) => {
        timer = setTimeout(() => reject(new Error('update_check_timeout')), milliseconds);
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
}
