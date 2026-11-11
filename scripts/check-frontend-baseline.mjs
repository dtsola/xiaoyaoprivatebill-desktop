#!/usr/bin/env node
// 校验前端冻结基线：自 `web-baseline` 以来只出现允许的改动。
//
// 用法：node scripts/check-frontend-baseline.mjs [ref]
// 环境变量：BASELINE_REF（默认 web-baseline）

import { execFileSync } from 'node:child_process'
import process from 'node:process'

import { violations } from './frontend-baseline.mjs'

const base = process.argv[2] || process.env.BASELINE_REF || 'web-baseline'

function git(args) {
  return execFileSync('git', args, { encoding: 'utf8' })
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
}

const changed = new Set([
  ...git(['diff', '--name-only', base, '--', 'frontend']),
  ...git(['ls-files', '--others', '--exclude-standard', '--', 'frontend'])
])

const bad = violations([...changed])
if (bad.length > 0) {
  console.error('[FAIL] 前端出现越界改动：')
  for (const path of bad) console.error(`  - ${path}`)
  console.error('只允许修改 8 个基线文件，以及离线图标/测试配置。')
  process.exit(1)
}

console.log(`[OK] 前端改动均在允许范围内（${changed.size} 个文件，基线 ${base}）`)
