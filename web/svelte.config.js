import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: vitePreprocess(),
  kit: {
    // SPA: the Rust server embeds `build/` and falls back to index.html for client routes.
    adapter: adapter({ fallback: 'index.html', strict: false }),
    alias: { $fixtures: 'fixtures' }
  }
};

export default config;
