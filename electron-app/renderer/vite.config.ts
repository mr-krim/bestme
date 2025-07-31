/// <reference types="vitest" />
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import sveltePreprocess from 'svelte-preprocess'

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [svelte({ 
    preprocess: sveltePreprocess({ typescript: true })
  })],

  test: {
    globals: true,
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{js,ts}'],
    deps: {
      inline: ['svelte-toast'] 
    }
  },

  // Electron-specific configuration
  clearScreen: false,
  server: {
    port: 3000,
    strictPort: true,
  },
  // Use relative base for Electron
  base: './',
  build: {
    // Modern browser target for Electron 37.x (Chrome 130)
    target: 'chrome130',
    outDir: 'dist',
    minify: process.env.NODE_ENV === 'production' ? 'esbuild' : false,
    sourcemap: process.env.NODE_ENV === 'development',
    rollupOptions: {
      input: {
        main: 'index.html'
      }
    }
  },
}))
