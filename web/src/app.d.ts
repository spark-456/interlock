/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** `1` serves every API call and SSE stream from web/fixtures/. */
  readonly VITE_FIXTURES?: string;
  /** Fixture replay speed multiplier (default 1). */
  readonly VITE_FIXTURE_SPEED?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}

declare global {
  namespace App {}
}

export {};
