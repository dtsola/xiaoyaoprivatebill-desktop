import { defineConfig } from '@playwright/test'

// UI 自动验收：跑 Vite Web 版，Tauri IPC 由 e2e/fixtures/tauri-mock.js 拦截。
// 使用系统 Edge，避免下载 Chromium。
export default defineConfig({
  testDir: './e2e',
  // 每个可交付功能的验收产物独立存放，默认不入库。
  outputDir: process.env.PW_OUTPUT_DIR || 'test-results',
  timeout: 30_000,
  fullyParallel: false,
  retries: 0,
  reporter: [['list']],
  workers: process.env.CI ? 2 : 4,
  use: {
    baseURL: 'http://localhost:41420',
    browserName: 'chromium',
    channel: 'msedge',
    viewport: { width: 1280, height: 840 },
    locale: 'zh-CN',
    trace: 'retain-on-failure',
    screenshot: 'on'
  },
  webServer: {
    command: 'npm --prefix frontend run dev',
    url: 'http://localhost:41420',
    reuseExistingServer: !process.env.CI,
    timeout: 120_000
  }
})
