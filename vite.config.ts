import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [
    vue(),
    tailwindcss(),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    port: 5173,
    strictPort: true,
    // Cargo writes build artifacts while the dev server runs, and watching
    // them throws EBUSY on Windows. Ignore the whole Rust tree plus the
    // directories Vite would otherwise watch by default.
    watch: {
      ignored: [
        '**/node_modules/**',
        '**/.git/**',
        '**/target/**',
        '**/src-tauri/**',
      ],
    },
  },
  build: {
    outDir: 'src-tauri/dist',
    emptyOutDir: true,
  },
  test: {
    environment: 'happy-dom',
    globals: true,
    include: ['src/**/*.test.ts', 'src/**/*.test.tsx'],
  },
})
