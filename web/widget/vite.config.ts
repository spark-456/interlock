import { defineConfig } from 'vite';

// The widget is its own tiny bundle (no framework); il-publish embeds the output and serves it at /w/:app.js.
export default defineConfig({
  root: import.meta.dirname,
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    lib: { entry: 'widget.ts', formats: ['iife'], name: 'InterlockWidget', fileName: () => 'widget.js' }
  }
});
