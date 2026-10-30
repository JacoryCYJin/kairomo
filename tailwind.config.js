/** @type {import('tailwindcss').Config} */
export default {
  content: ['./index.html', './src/**/*.{vue,ts}'],
  theme: {
    extend: {
      colors: Object.fromEntries(
        [
          'paper',
          'surface',
          'surface-muted',
          'hover',
          'selected',
          'ink',
          'muted',
          'faint',
          'line',
          'line-strong',
          'accent',
          'rest',
          'danger',
          'danger-soft',
          'dialog-confirm',
          'dialog-confirm-ink',
        ].map((name) => [name, `var(--${name})`]),
      ),
      fontFamily: {
        sans: ['-apple-system', 'BlinkMacSystemFont', 'Segoe UI', 'PingFang SC', 'sans-serif'],
        mono: ['SFMono-Regular', 'Consolas', 'monospace'],
      },
      fontSize: {
        timer: ['4.75rem', { lineHeight: '1.25', letterSpacing: '-0.1875rem' }],
        'timer-sign': ['1.9375rem', { lineHeight: '1' }],
      },
      screens: { wide: '821px' },
      boxShadow: { dialog: 'var(--dialog-shadow)' },
    },
  },
  plugins: [],
};
