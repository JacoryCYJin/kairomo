import mountain from './assets/mountain.png';
import focus from './assets/focus-lower.png';
import rest from './assets/rest-lower.png';
import climber from './assets/climber.png';
export const snowAssets = { mountain, focus, rest, climber };
const pending = new Map<string, Promise<void>>();
/** Reuse the exact production assets for previews and native windows. */
export function prepareSnowAssets(urls: readonly string[]) {
  return Promise.all(
    urls.map((url) => {
      let result = pending.get(url);
      if (!result) {
        result = new Promise<void>((resolve, reject) => {
          const image = new Image();
          image.onload = () => resolve();
          image.onerror = () => {
            pending.delete(url);
            reject(new Error('Snow theme asset unavailable'));
          };
          image.src = url;
          if (image.complete && image.naturalWidth > 0) resolve();
        });
        pending.set(url, result);
      }
      return result;
    }),
  );
}
