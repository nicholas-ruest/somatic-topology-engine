import { defineConfig } from 'vite'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss()],
  build: { sourcemap: false, target: 'es2022', assetsInlineLimit: 4096 },
  server: { host: '127.0.0.1', strictPort: true },
  preview: { host: '127.0.0.1', strictPort: true },
  test: { environment: 'jsdom', setupFiles: ['./src/test/setup.js'] },
})
