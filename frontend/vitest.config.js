import { defineConfig } from 'vitest/config'

// 纯逻辑/传输层单测：node 环境足够，不加载 DOM。
export default defineConfig({
  test: {
    environment: 'node',
    include: ['src/**/*.{test,spec}.js']
  }
})
