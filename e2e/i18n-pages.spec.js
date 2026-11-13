import fs from 'node:fs'
import path from 'node:path'

import { expect, test } from '@playwright/test'
import { installTauriMock } from './fixtures/tauri-mock.js'

const read = (name) =>
  JSON.parse(fs.readFileSync(path.join('fixtures', 'expected', name), 'utf-8'))

const fixtures = {
  '/available_years': read('available_years.json'),
  '/available_dates': read('available_dates.json'),
  '/categories': read('categories.json'),
  '/category_available_dates': read('category_available_dates.json'),
  '/yearly_analysis': read('yearly_analysis_2024.json'),
  '/monthly_analysis': read('monthly_analysis_2024_12.json'),
  '/category_analysis': read('category_analysis_2024.json'),
  '/category_analysis_detail': read('category_analysis_2024_food.json'),
  '/time_analysis': read('time_analysis_2024.json'),
  '/daily_data': read('daily_data_2024.json'),
  '/analysis': read('analysis_2024.json'),
  '/transactions': read('transactions_expense.json'),
  '/files': read('files_empty.json')
}

// 每个路由的英文标题与必须出现的英文关键词（覆盖页面级与卡片级文案）
const PAGES = [
  {
    hash: '#/yearly',
    name: 'yearly',
    title: 'Yearly Overview',
    checks: ['Yearly spending', 'Spending trend', 'Spending by category', 'Housing']
  },
  {
    hash: '#/monthly',
    name: 'monthly',
    title: 'Monthly',
    checks: ['Monthly spending', 'Income & spending trend', 'Spending by category', 'Housing']
  },
  {
    hash: '#/category',
    name: 'category',
    title: 'Categories',
    checks: ['Spending share', 'Spending trend', 'Amount distribution', 'Housing']
  },
  {
    hash: '#/time',
    name: 'time',
    title: 'Time Analysis',
    checks: ['Daily spending heatmap', 'Hourly distribution', 'Weekday / weekend']
  },
  {
    hash: '#/insights',
    name: 'insights',
    title: 'Insights',
    checks: ['Money flow', 'Spending profile', 'Deep insights', 'Spending scenarios']
  },
  {
    hash: '#/transactions',
    name: 'transactions',
    title: 'Transactions',
    checks: ['Transaction details', 'Description', 'Counterparty', 'Status', 'Transport']
  },
  {
    hash: '#/settings',
    name: 'settings',
    title: 'Settings',
    checks: ['Bill files', 'Data management', 'Storage & audit', 'Desktop', 'Language']
  },
  {
    hash: '#/agent',
    name: 'agent',
    title: 'AI Agent',
    checks: ['Status', 'Endpoint', 'Enable MCP service', 'Connect XiaoyaoClaw']
  },
  {
    hash: '#/about-author',
    name: 'about',
    title: 'About',
    checks: ['Contact', 'About this project', 'Official website']
  }
]

test('英文界面覆盖全部页面', async ({ page }) => {
  const errors = []
  page.on('pageerror', (error) => errors.push(error.message))
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text())
  })

  await page.addInitScript(installTauriMock, fixtures)
  await page.setViewportSize({ width: 1440, height: 1000 })

  await page.goto('/#/settings')
  await page.waitForTimeout(800)
  await page.locator('.language-select').selectOption('en-US')
  await expect(page.locator('.nav-menu .nav-item').first()).toContainText('Home')

  fs.mkdirSync('artifacts/ui/i18n', { recursive: true })

  for (const item of PAGES) {
    // 使用 hash 路由在同一页面内切换，避免刷新后丢失手动选择的语言
    await page.evaluate((hash) => {
      window.location.hash = hash
    }, item.hash)
    await expect(page.locator('.page-title').first()).toHaveText(item.title, {
      timeout: 10_000
    })
    await page.waitForTimeout(900)

    const body = page.locator('body')
    for (const text of item.checks) {
      await expect(body).toContainText(text, { timeout: 10_000 })
    }

    if (item.hash === '#/insights') {
      // 年度故事幻灯片是 innerHTML 模板，单独验证英文首屏
      await page.locator('.story-entry-card').click()
      await expect(page.locator('.slide.active h1')).toHaveText(
        'Your yearly spending story'
      )
      await page.locator('.close-story').click()
    }

    await page.screenshot({
      path: `artifacts/ui/i18n/${item.name}-en.png`,
      fullPage: true
    })
  }

  expect(errors, `控制台错误: ${errors.join(' | ')}`).toEqual([])
})
