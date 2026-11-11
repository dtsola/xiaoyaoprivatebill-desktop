import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

// 桌面端固定端口：41420 位于动态端口保留区之外，避免 1420/7420/5420 被 Hyper-V/WinNAT 占用。
export default defineConfig({
  plugins: [vue()],
  base: './',
  clearScreen: false,
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    port: 41420,
    strictPort: true,
    watch: {
      ignored: ['**/src-tauri/**']
    }
  },
  build: {
    outDir: 'dist',
    assetsDir: 'assets'
  }
})
