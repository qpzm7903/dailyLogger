import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue(), tailwindcss()],
  test: {
    environment: 'jsdom',
    globals: true,
    include: ['src/**/*.{spec,test}.{js,ts}', 'src/__tests__/**/*.{js,ts}'],
    setupFiles: ['src/setupTests.ts'],
  },
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: true,  // Listen on all addresses
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  build: {
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        'quick-note': resolve(__dirname, 'quick-note.html'),
      },
    },
  }
})
