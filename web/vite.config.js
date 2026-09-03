import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import tailwindcss from '@tailwindcss/vite';

// Vite configuration for the Noob CompressorLab SPA.
//
// Production: `vite build` writes `dist/`, which the Rust side serves from
// disk (the standalone binary, `assets_dir`) or embeds in the plug-in
// binary (`include_dir!` under `--features plugin`). Build the SPA before
// building the plug-in, or `include_dir!` has nothing to embed.
//
// Development: `vite` hot-reloads the SPA and proxies the WebSocket and the
// discovery endpoints to a running noob-vst-webgui-framework server (`NOOB_VST_WEBGUI_FRAMEWORK_PORT`, default
// 4244, the standalone's preferred port; if that port was taken the
// standalone moved up and printed the port it got):
//
//     cargo run --bin noob-compressorlab-standalone      # terminal 1
//     NOOB_VST_WEBGUI_FRAMEWORK_PORT=4244 npm run dev                     # terminal 2, in web/
const serverPort = process.env.NOOB_VST_WEBGUI_FRAMEWORK_PORT || '4244';

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  // Relative asset URLs: the page must work from whatever origin and port
  // the plug-in's server ends up on, and from an embedded web view.
  base: './',
  resolve: {
    // The framework's Vue layer and this app must share one copy of `vue`,
    // which reactivity requires.
    dedupe: ['vue'],
  },
  // The framework is installed from git; keep it out of the dependency
  // pre-bundle so an `npm link`ed checkout hot-reloads instead of being frozen
  // into node_modules/.vite at start-up.
  optimizeDeps: { exclude: ['@noob-audio-engineering/noob-vst-webgui-framework'] },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    // The page only ever runs in a current WebView2 / WebKit / Chromium.
    target: 'es2022',
    // The plug-in embeds `dist/` byte for byte; keep it small.
    sourcemap: false,
  },
  server: {
    port: 5175,
    strictPort: false,
    proxy: {
      // The plug-in's WebSocket, so the page can talk to the real DSP.
      '/ws': { target: `ws://127.0.0.1:${serverPort}`, ws: true },
      // `/instance` and `/instances` (prefix match), used by the instance menu.
      '/instance': { target: `http://127.0.0.1:${serverPort}` },
    },
  },
});
