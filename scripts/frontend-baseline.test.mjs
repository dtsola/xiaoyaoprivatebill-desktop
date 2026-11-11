import assert from 'node:assert/strict'
import test from 'node:test'

import { isAllowed, violations } from './frontend-baseline.mjs'

test('允许的 8 个文件通过', () => {
  assert.equal(isAllowed('frontend/src/api/client.js'), true)
  assert.equal(isAllowed('frontend/src/views/Settings.vue'), true)
  assert.equal(isAllowed('frontend/src/views/Home.vue'), true)
})

test('越界的页面组件被拦截', () => {
  assert.deepEqual(violations(['frontend/src/views/Home.vue2']), [
    'frontend/src/views/Home.vue2'
  ])
  assert.deepEqual(violations(['frontend/src/stores/data.js']), [
    'frontend/src/stores/data.js'
  ])
})

test('图标迁移涉及的页面与公共组件放行', () => {
  assert.equal(isAllowed('frontend/src/views/Yearly.vue'), true)
  assert.equal(isAllowed('frontend/src/components/common/Toast.vue'), true)
  assert.equal(isAllowed('frontend/src/assets/styles/main.css'), true)
})

test('离线图标、测试配置与测试文件不算越界', () => {
  assert.equal(isAllowed('frontend/public/vendor/fontawesome/css/all.min.css'), true)
  assert.equal(isAllowed('frontend/vitest.config.js'), true)
  assert.equal(isAllowed('frontend/src/api/client.test.js'), true)
})

test('品牌资源替换不算越界', () => {
  assert.equal(isAllowed('frontend/public/images/logo_128.png'), true)
  assert.equal(isAllowed('frontend/public/favicon.png'), true)
  assert.deepEqual(violations(['frontend/public/images/logo_512.png']), [
    'frontend/public/images/logo_512.png'
  ])
})

test('非前端路径一律通过', () => {
  assert.deepEqual(violations(['crates/bill-api/src/lib.rs', 'docs/开发进度.md']), [])
})
