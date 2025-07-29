/// <reference types="vitest" />
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import sveltePreprocess from 'svelte-preprocess'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [svelte({ 
    preprocess: sveltePreprocess({ typescript: true }) // Enable TypeScript preprocessing
  })],

  test: {
    globals: true,
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{js,ts}'],
    deps: {
      // Ensure svelte-toast is processed by Vitest's transforms
      inline: ['svelte-toast'] 
    }
  },

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 5173,
    strictPort: false,
  },
  // to make use of `TAURI_DEBUG` and other env variables
  // https://tauri.studio/v1/api/config#buildconfig.beforedevcommand
  envPrefix: ['VITE_', 'TAURI_'],
  // Use relative base for Tauri
  base: './',
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
  },
}))
