import { describe, expect, it } from 'vitest'

import {
  AMOUNT_FIELDS,
  DEFAULT_AMOUNT_VISIBILITY,
  POSTER_HEIGHT,
  POSTER_THEMES,
  POSTER_WIDTH,
  PRODUCT_SITE_URL,
  buildPosterModel,
  defaultAmountVisibility,
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

const monthly = {
  stats: {
    total_expense: 27279.53,
    total_income: 19978.44,
    balance: -7301.09,
    expense_count: 135,
    income_count: 18
  },
  daily_data: {
    expense: {
      dates: Array.from({ length: 31 }, (_, index) => `2024-12-${String(index + 1).padStart(2, '0')}`),
      amounts: Array.from({ length: 31 }, (_, index) => (index === 11 ? 5000 : 100))
    }
  },
  categories_source: {
    expense: [
      { 交易分类: '住房物业', 来源: '支付宝', 金额: 5741.09 },
      { 交易分类: '餐饮美食', 来源: '微信', 金额: 3000 }
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

  it('按开关显示金额或掩码', () => {
    expect(displayAmount(336760.51, true)).toBe('336,760.51')
    expect(displayAmount(336760.51, false)).toBe('••••••')
  })
})

describe('年度模型', () => {
  const model = buildPosterModel(yearly, { period: 'year', year: 2024 })

  it('汇总年度指标与 12 个月趋势', () => {
    expect(model.period).toBe('year')
    expect(model.periodLabel).toBe('2024 年')
    expect(model.heroLabel).toBe('年度支出')
    expect(model.expense).toBeCloseTo(336760.51)
    expect(model.income).toBeCloseTo(205791.15)
    expect(model.balance).toBeCloseTo(-130969.36)
    expect(model.chart.items).toHaveLength(12)
    expect(model.chart.items[0]).toEqual({ label: '1', value: 35033.34 })
    expect(model.chart.peak).toEqual({ label: '3', value: 67404.67 })
    expect(model.chart.labelEvery).toBe(1)
  })

  it('分类按金额合并后取前五并计算占比', () => {
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
    const empty = buildPosterModel({ yearly_stats: {} }, { period: 'year', year: 2023 })
    expect(empty.year).toBe('2023')
    expect(empty.topCategories).toEqual([])
    expect(empty.chart.items).toEqual([])
    expect(empty.expense).toBe(0)
  })

  it('回退到 categories.expense 结构', () => {
    const fallback = buildPosterModel(
      {
        yearly_stats: { total_expense: 300 },
        categories: { expense: { names: ['餐饮美食', '交通出行'], amounts: [100, 200] } }
      },
      { period: 'year', year: 2024 }
    )
    expect(fallback.topCategories[0]).toEqual({ name: '交通出行', amount: 200, ratio: 66.7 })
  })
})

describe('月度模型', () => {
  const model = buildPosterModel(monthly, { period: 'month', year: 2024, month: 12 })

  it('使用月度指标与每日趋势', () => {
    expect(model.period).toBe('month')
    expect(model.periodLabel).toBe('2024 年 12 月')
    expect(model.heroLabel).toBe('本月支出')
    expect(model.incomeLabel).toBe('本月收入')
    expect(model.chartTitle).toBe('每日支出')
    expect(model.expense).toBeCloseTo(27279.53)
    expect(model.totalCount).toBe(153)
    expect(model.activeDays).toBe(31)
    expect(model.chart.items).toHaveLength(31)
    expect(model.chart.peak).toEqual({ label: '12', value: 5000 })
    expect(model.chart.labelEvery).toBe(5)
  })

  it('月度分类占比基于本月分类合计', () => {
    expect(model.topCategories[0].name).toBe('住房物业')
    expect(model.topCategories[0].ratio).toBeCloseTo(65.7, 1)
  })

  it('当月数据未覆盖月末时给出截至日期', () => {
    const partial = {
      stats: { total_expense: 500, total_income: 0 },
      daily_data: {
        expense: {
          dates: Array.from({ length: 31 }, (_, index) => `2024-12-${String(index + 1).padStart(2, '0')}`),
          amounts: Array.from({ length: 31 }, (_, index) => (index < 12 ? 10 : 0))
        },
        income: { dates: [], amounts: [] }
      }
    }
    const current = buildPosterModel(partial, {
      period: 'month',
      year: 2024,
      month: 12,
      today: new Date(2024, 11, 12)
    })
    expect(current.asOfLabel).toBe('数据截至 12 日')

    const past = buildPosterModel(partial, {
      period: 'month',
      year: 2024,
      month: 12,
      today: new Date(2025, 0, 5)
    })
    expect(past.asOfLabel).toBe('')

    const full = buildPosterModel(monthly, {
      period: 'month',
      year: 2024,
      month: 12,
      today: new Date(2024, 11, 31)
    })
    expect(full.asOfLabel).toBe('')
  })

  it('没有每日数据时降级为空趋势', () => {
    const empty = buildPosterModel(
      { stats: { total_expense: 100 } },
      { period: 'month', year: 2024, month: 5 }
    )
    expect(empty.chart.items).toEqual([])
    expect(empty.periodLabel).toBe('2024 年 5 月')
  })
})

describe('模板与金额开关', () => {
  it('内置八套模板且 id 唯一', () => {
    expect(POSTER_THEMES).toHaveLength(8)
    expect(new Set(POSTER_THEMES.map((theme) => theme.id)).size).toBe(8)
    expect(POSTER_THEMES.map((theme) => theme.name)).toEqual([
      '晴空',
      '墨夜',
      '暖阳',
      '薄荷',
      '樱花',
      '深海',
      '拿铁',
      '极简'
    ])
    POSTER_THEMES.forEach((theme) => {
      expect(theme.page).toHaveLength(3)
      expect(theme.palette).toHaveLength(5)
    })
  })

  it('金额开关覆盖全部可隐藏字段且默认全开', () => {
    expect(AMOUNT_FIELDS.map((field) => field.key)).toEqual([
      'expense',
      'income',
      'balance',
      'categories',
      'peak'
    ])
    expect(Object.values(DEFAULT_AMOUNT_VISIBILITY).every(Boolean)).toBe(true)
  })

  it('月度默认隐藏收入与结余，年度全部显示', () => {
    expect(defaultAmountVisibility('year')).toEqual(DEFAULT_AMOUNT_VISIBILITY)
    expect(defaultAmountVisibility('month')).toEqual({
      ...DEFAULT_AMOUNT_VISIBILITY,
      income: false,
      balance: false
    })
  })

  it('导出常量与官网地址', () => {
    expect(POSTER_WIDTH).toBe(1080)
    expect(POSTER_HEIGHT).toBe(1440)
    expect(PRODUCT_SITE_URL).toBe('https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/')
  })
})
