/**
 * Noob CompressorLab entry point. Vite serves this file in development
 * and bundles it into `dist/` for production, where the standalone binary
 * serves it from disk and the plug-in embeds it (`include_dir!` under
 * `--features plugin`).
 *
 * In development the client is told about the offline design manifest
 * (`dev/manifest.js`): if no real server answers within a second the page
 * renders against synthetic parameters and frames, and hands over to the
 * plug-in the moment it connects. Production builds never include it.
 */
import { createApp } from 'vue';
import { configureClient } from '@noob-audio-engineering/noob-vst-webgui-framework/vue';
import './style.css';
import App from './App.vue';

if (import.meta.env.DEV) {
  const { offline } = await import('./dev/manifest.js');
  configureClient({ offline });
}

createApp(App).mount('#app');
