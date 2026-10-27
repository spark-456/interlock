import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vitest/config';

const backend = process.env.IL_BACKEND ?? 'http://127.0.0.1:8080';
const proxied = ['/api', '/v1', '/mcp', '/hooks', '/w', '/healthz'];

export default defineConfig({
  plugins: [sveltekit()],
  server: {
    port: 5173,
    proxy: process.env.VITE_FIXTURES === '1' ? undefined : Object.fromEntries(proxied.map((p) => [p, backend]))
  },
  test: {
    include: ['src/**/*.test.ts', 'widget/**/*.test.ts'],
    environment: 'node'
  }
});
