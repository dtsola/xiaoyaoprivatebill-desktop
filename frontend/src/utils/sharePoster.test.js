import { describe, expect, it } from 'vitest'

import {
  PRODUCT_SITE_URL,
  POSTER_HEIGHT,
  POSTER_WIDTH,
  buildPosterModel,
  displayAmount,
  formatCompactYuan,
  formatYuan
} from './sharePoster'

const yearly = {
  yearly_stats: {
    total_expense: 336760.51,
    total_income: 205791.15,
    balance: -130969.36,
    total_count: 1838,
    expense_count: 1689,
    active_days: 364,
    avg_daily_expense: 925.17
  },
  trends: {
    months: [
      '2024-01',
      '2024-02',
      '2024-03',
      '2024-04',
      '2024-05',
      '2024-06',
      '2024-07',
      '2024-08',
      '2024-09',
      '2024-10',
      '2024-11',
      '2024-12'
    ],
    expenses: [35033.34, 22314.14, 67404.67, 26879.6, 25629.01, 26820.41, 31109.62, 19872.5, 21919.08, 28638.13, 39403.28, 27279.53],
    incomes: Array(12).fill(0)
  },
  categories_source: {
    expense: [
      { 交易分类: '餐饮美食', 来源: '支付宝', 金额: 1000 },
      { 交易分类: '住房物业', 来源: '支付宝', 金额: 5000 },
      { 交易分类: '餐饮美食', 来源: '微信', 金额: 500 },
      { 交易分类: '交通出行', 来源: '微信', 金额: 2000 },
      { 交易分类: '服饰装扮', 来源: '支付宝', 金额: 3000 },
      { 交易分类: '数码电器', 来源: '支付宝', 金额: 1000 },
      { 交易分类: '文化休闲', 来源: '微信', 金额: 500 }
    ]
  }
}

describe('格式化', () => {
  it('金额按千分位保留两位', () => {
    expect(formatYuan(336760.51)).toBe('336,760.51')
    expect(formatYuan(0)).toBe('0.00')
    expect(formatYuan(null)).toBe('0.00')
  })

  it('分类金额用万/亿缩写', () => {
    expect(formatCompactYuan(17374.89)).toBe('1.74万')
    expect(formatCompactYuan(9999.5)).toBe('9,999.50')
    expect(formatCompactYuan(120000000)).toBe('1.20亿')
  })

  it('隐藏金额时只输出掩码', () => {
    expect(displayAmount(336760.51, false)).toBe('336,760.51')
    expect(displayAmount(336760.51, true)).toBe('••••••')
  })
})

describe('海报数据模型', () => {
  it('汇总年度指标与 12 个月趋势', () => {
    const model = buildPosterModel(yearly, 2024)
    expect(model.year).toBe('2024')
    expect(model.expense).toBeCloseTo(336760.51)
    expect(model.income).toBeCloseTo(205791.15)
    expect(model.balance).toBeCloseTo(-130969.36)
    expect(model.months).toHaveLength(12)
    expect(model.months[0]).toEqual({ label: '1月', value: 35033.34 })
    expect(model.peakMonth).toEqual({ label: '3月', value: 67404.67 })
  })

  it('分类按金额合并后取前五并计算占比', () => {
    const model = buildPosterModel(yearly, 2024)
    expect(model.topCategories.map((item) => item.name)).toEqual([
      '住房物业',
      '服饰装扮',
      '交通出行',
      '餐饮美食',
      '数码电器'
    ])
    expect(model.topCategories[0].ratio).toBeCloseTo(38.5, 1)
    expect(model.topCategories[3].amount).toBeCloseTo(1500)
  })

  it('缺少分类与趋势时不抛错', () => {
    const model = buildPosterModel({ yearly_stats: {} }, 2023)
    expect(model.year).toBe('2023')
    expect(model.topCategories).toEqual([])
    expect(model.months).toEqual([])
    expect(model.expense).toBe(0)
  })

  it('回退到 categories.expense 结构', () => {
    const model = buildPosterModel(
      {
        yearly_stats: { total_expense: 300 },
        categories: { expense: { names: ['餐饮美食', '交通出行'], amounts: [100, 200] } }
      },
      2024
    )
    expect(model.topCategories[0]).toEqual({ name: '交通出行', amount: 200, ratio: 66.7 })
  })

  it('导出常量与官网地址', () => {
    expect(POSTER_WIDTH).toBe(1080)
    expect(POSTER_HEIGHT).toBe(1440)
    expect(PRODUCT_SITE_URL).toBe('https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/')
  })
})
