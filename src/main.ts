import { createApp } from 'vue';
import App from './app/App.vue';
import './shared/styles/tokens.css';
import './shared/styles/base.css';
if (import.meta.env.DEV && new URLSearchParams(location.search).get('dev') === 'series') {
  void import('./dev/SeriesPreview.vue').then(({ default: Preview }) =>
    createApp(Preview).mount('#app'),
  );
} else createApp(App).mount('#app');
