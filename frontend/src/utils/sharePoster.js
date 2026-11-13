import { localizeCategory } from '../i18n/categories'

/**
 * 账单分享海报：数据模型、模板与 Canvas 绘制。
 *
 * 海报固定 1080×1440（3:4，适合朋友圈 / 小红书 / 微博 / 抖音），完全在本地绘制，
 * 不发起任何网络请求；二维码由 Rust 侧离线生成。
 */

export const POSTER_WIDTH = 1080
export const POSTER_HEIGHT = 1440
export const PRODUCT_SITE_URL = 'https://www.xiaoyaosai.com/xiaoyaoprivatebill-desktop/'
export const PRODUCT_NAME = '小遥账单桌面版'
export const PRODUCT_SLOGAN = '隐私优先 · 完全离线 · 本地分析'

const FONT_STACK = '"PingFang SC", "Microsoft YaHei", "Helvetica Neue", Arial, sans-serif'

/**
 * 海报模板：只改配色与氛围，不改变信息结构。
 */
export const POSTER_THEMES = [
  {
    id: 'sky',
    name: '晴空',
    description: '清爽品牌蓝',
    swatch: ['#E7F1FF', '#007AFF'],
    page: ['#E7F1FF', '#F1F7FF', '#FBFDFF'],
    card: '#FFFFFF',
    ink: '#0B1B33',
    soft: '#5B6B84',
    muted: '#8E9AAF',
    brand: '#007AFF',
    green: '#34C759',
    red: '#FF453A',
    panel: '#F7F9FC',
    track: '#EDF2FA',
    line: '#E6EDF7',
    bar: '#C9DDFB',
    pillBg: '#EAF3FF',
    pillInk: '#007AFF',
    glow1: 'rgba(0, 122, 255, 0.16)',
    glow2: 'rgba(88, 86, 214, 0.10)',
    shadow: 'rgba(11, 27, 51, 0.12)',
    palette: ['#007AFF', '#5856D6', '#FF9500', '#34C759', '#FF375F']
  },
  {
    id: 'ink',
    name: '墨夜',
    description: '深色高级感',
    swatch: ['#0B1528', '#4DA3FF'],
    page: ['#0A1226', '#0F1B33', '#0B1528'],
    card: '#111E38',
    ink: '#F5F8FF',
    soft: '#C3CFE4',
    muted: '#8496B3',
    brand: '#4DA3FF',
    green: '#4ADE80',
    red: '#FF7A7A',
    panel: '#16294A',
    track: '#1E355C',
    line: '#22375C',
    bar: '#2B4A7A',
    pillBg: '#1B3560',
    pillInk: '#8CC4FF',
    glow1: 'rgba(77, 163, 255, 0.24)',
    glow2: 'rgba(124, 92, 255, 0.18)',
    shadow: 'rgba(0, 0, 0, 0.42)',
    palette: ['#4DA3FF', '#8A7CFF', '#FFB020', '#4ADE80', '#FF7AA2']
  },
  {
    id: 'sunset',
    name: '暖阳',
    description: '温暖杂志风',
    swatch: ['#FFF3E6', '#E8663D'],
    page: ['#FFF3E6', '#FFF8F0', '#FFFCF7'],
    card: '#FFFDFB',
    ink: '#3A2416',
    soft: '#7A5A45',
    muted: '#A88973',
    brand: '#E8663D',
    green: '#3D9A6A',
    red: '#D14B3F',
    panel: '#FFF6EC',
    track: '#F5E7D8',
    line: '#F0E0CF',
    bar: '#F3C9A8',
    pillBg: '#FFE9D6',
    pillInk: '#C9552F',
    glow1: 'rgba(232, 102, 61, 0.16)',
    glow2: 'rgba(247, 181, 86, 0.18)',
    shadow: 'rgba(58, 36, 22, 0.14)',
    palette: ['#E8663D', '#C9552F', '#F0A02A', '#3D9A6A', '#C74B7C']
  },
  {
    id: 'mint',
    name: '薄荷',
    description: '清新青绿',
    swatch: ['#E8F7F1', '#10A37F'],
    page: ['#E8F7F1', '#F2FBF7', '#FAFDFB'],
    card: '#FFFFFF',
    ink: '#0D2B23',
    soft: '#3F6B5C',
    muted: '#7AA093',
    brand: '#10A37F',
    green: '#16A34A',
    red: '#E5484D',
    panel: '#F1FAF6',
    track: '#E3F2EC',
    line: '#DDEBE5',
    bar: '#BFE8D9',
    pillBg: '#E2F6EE',
    pillInk: '#0E8C6C',
    glow1: 'rgba(16, 163, 127, 0.16)',
    glow2: 'rgba(56, 189, 248, 0.12)',
    shadow: 'rgba(13, 43, 35, 0.12)',
    palette: ['#10A37F', '#0EA5E9', '#F59E0B', '#22C55E', '#F472B6']
  },
  {
    id: 'sakura',
    name: '樱花',
    description: '柔粉少女感',
    swatch: ['#FFF0F5', '#EC4899'],
    page: ['#FFF0F5', '#FFF6F9', '#FFFCFD'],
    card: '#FFFFFF',
    ink: '#3A1E2A',
    soft: '#7A4E60',
    muted: '#A87D8E',
    brand: '#EC4899',
    green: '#16A34A',
    red: '#E11D48',
    panel: '#FFF4F8',
    track: '#FBE4EE',
    line: '#F6DCE7',
    bar: '#F9C6DC',
    pillBg: '#FDE7F1',
    pillInk: '#DB2777',
    glow1: 'rgba(236, 72, 153, 0.16)',
    glow2: 'rgba(251, 146, 60, 0.12)',
    shadow: 'rgba(58, 30, 42, 0.12)',
    palette: ['#EC4899', '#8B5CF6', '#F59E0B', '#22C55E', '#F43F5E']
  },
  {
    id: 'ocean',
    name: '深海',
    description: '青蓝夜色',
    swatch: ['#06222B', '#22D3EE'],
    page: ['#06222B', '#08303C', '#062730'],
    card: '#0C2833',
    ink: '#EAF7FB',
    soft: '#B7D6DF',
    muted: '#7FA3AE',
    brand: '#22D3EE',
    green: '#34D399',
    red: '#FB7185',
    panel: '#11323E',
    track: '#173F4D',
    line: '#194553',
    bar: '#1E4E5E',
    pillBg: '#103B48',
    pillInk: '#7DE7F7',
    glow1: 'rgba(34, 211, 238, 0.22)',
    glow2: 'rgba(45, 212, 191, 0.16)',
    shadow: 'rgba(0, 0, 0, 0.45)',
    palette: ['#22D3EE', '#2DD4BF', '#FBBF24', '#34D399', '#F472B6']
  },
  {
    id: 'latte',
    name: '拿铁',
    description: '温暖咖啡棕',
    swatch: ['#F6EFE7', '#B07D4F'],
    page: ['#F6EFE7', '#FBF6F0', '#FEFCF9'],
    card: '#FFFDFA',
    ink: '#2E241B',
    soft: '#6B5A4A',
    muted: '#9C8B7A',
    brand: '#B07D4F',
    green: '#4D7C0F',
    red: '#B91C1C',
    panel: '#FAF4ED',
    track: '#F0E6DA',
    line: '#EADFD2',
    bar: '#E3CDB6',
    pillBg: '#F4E8DA',
    pillInk: '#8C5A2E',
    glow1: 'rgba(176, 125, 79, 0.16)',
    glow2: 'rgba(217, 119, 6, 0.12)',
    shadow: 'rgba(46, 36, 27, 0.14)',
    palette: ['#B07D4F', '#8B5E34', '#D97706', '#4D7C0F', '#9F1239']
  },
  {
    id: 'mono',
    name: '极简',
    description: '黑白留白',
    swatch: ['#F2F2F4', '#111114'],
    page: ['#F2F2F4', '#F7F7F9', '#FCFCFD'],
    card: '#FFFFFF',
    ink: '#111114',
    soft: '#4B4B52',
    muted: '#8A8A93',
    brand: '#111114',
    green: '#2F7D5A',
    red: '#B42318',
    panel: '#F5F5F7',
    track: '#E8E8EC',
    line: '#E2E2E6',
    bar: '#C9C9D1',
    pillBg: '#EAEAEE',
    pillInk: '#3A3A42',
    glow1: 'rgba(17, 17, 20, 0.10)',
    glow2: 'rgba(120, 120, 140, 0.10)',
    shadow: 'rgba(17, 17, 20, 0.14)',
    palette: ['#111114', '#3F3F46', '#52525B', '#71717A', '#8A8A93']
  }
]

/** 海报上可独立开关的金额项。 */
export const AMOUNT_FIELDS = [
  { key: 'expense', label: '支出总额' },
  { key: 'income', label: '收入' },
  { key: 'balance', label: '结余' },
  { key: 'categories', label: '分类金额' },
  { key: 'peak', label: '趋势最高金额' }
]


/**
 * 海报画布文案。海报是图片，不走 vue-i18n，避免 Canvas 层依赖组件上下文。
 */
const POSTER_COPY = {
  'zh-CN': {
    appName: '小遥账单',
    product: '小遥账单桌面版',
    slogan: '隐私优先 · 完全离线 · 本地分析',
    titleYear: '年度账单',
    titleMonth: '月度账单',
    subtitle: '每一笔支出，都是这一年的注脚',
    heroYear: '年度支出',
    heroMonth: '本月支出',
    incomeYear: '年度收入',
    incomeMonth: '本月收入',
    balanceYear: '年度结余',
    balanceMonth: '本月结余',
    chartYear: '月度支出趋势',
    chartMonth: '每日支出',
    top: '消费 TOP 5',
    sort: '按支出金额排序',
    qrHint: '扫码了解详情',
    qrPlaceholder: '二维码',
    categoriesEmpty: '还没有分类数据',
    chartEmpty: '这段时间还没有支出数据',
    transactions: '交易笔数',
    periodLabel: (year, month, period) =>
      period === 'month' ? `${year} 年 ${month} 月` : `${year} 年`,
    asOf: (day) => `数据截至 ${day} 日`,
    dailyAvg: (value) => `日均消费 ¥${value}`,
    dailyAvgHidden: '日均消费 ••••',
    countText: (count) => `${count} 笔`,
    demoSuffix: ' · 示例',
    peakText: (label, value, period) =>
      `最高 ${label}${period === 'month' ? '日' : '月'}${value ? ` · ${value}` : ''}`
  },
  'en-US': {
    appName: 'Xiaoyao Bill',
    product: 'Xiaoyao Bill Desktop',
    slogan: 'Private · Fully offline · Local analysis',
    titleYear: 'Yearly Bill',
    titleMonth: 'Monthly Bill',
    subtitle: 'Every transaction tells a story of the year',
    heroYear: 'Yearly spending',
    heroMonth: 'Monthly spending',
    incomeYear: 'Yearly income',
    incomeMonth: 'Monthly income',
    balanceYear: 'Yearly balance',
    balanceMonth: 'Monthly balance',
    chartYear: 'Monthly spending trend',
    chartMonth: 'Daily spending',
    top: 'TOP 5 categories',
    sort: 'By spending amount',
    qrHint: 'Scan to learn more',
    qrPlaceholder: 'QR code',
    categoriesEmpty: 'No category data yet',
    chartEmpty: 'No spending in this period',
    transactions: 'Transactions',
    periodLabel: (year, month, period) =>
      period === 'month' ? `${year}-${String(month).padStart(2, '0')}` : `${year}`,
    asOf: (day) => `Data through day ${day}`,
    dailyAvg: (value) => `Daily average ¥${value}`,
    dailyAvgHidden: 'Daily average ••••',
    countText: (count) => `${count}`,
    demoSuffix: ' · Sample',
    peakText: (label, value, period) =>
      `Peak ${period === 'month' ? 'day ' : ''}${label}${value ? ` · ${value}` : ''}`
  }
}

export { CATEGORY_LABELS_EN, localizeCategory } from '../i18n/categories'

const posterCopy = (locale) => POSTER_COPY[locale === 'en-US' ? 'en-US' : 'zh-CN']

export const DEFAULT_AMOUNT_VISIBILITY = AMOUNT_FIELDS.reduce((result, field) => {
  result[field.key] = true
  return result
}, {})

/**
 * 各范围的金额默认显示策略。
 *
 * 月度海报更常出现在朋友圈等公开场合，默认隐藏收入与结余，用户可逐项打开。
 */
export function defaultAmountVisibility(period = 'year') {
  return {
    ...DEFAULT_AMOUNT_VISIBILITY,
    ...(period === 'month' ? { income: false, balance: false } : {})
  }
}

const toNumber = (value) => {
  const number = Number(value)
  return Number.isFinite(number) ? number : 0
}

/** 千分位金额，保留两位小数。 */
export function formatYuan(value) {
  return toNumber(value).toLocaleString('en-US', {
    minimumFractionDigits: 2,
    maximumFractionDigits: 2
  })
}

/**
 * 分类金额缩写：中文用「亿 / 万」，英文用 B / M / K；小数保持千分位。
 */
export function formatCompactYuan(value, locale = 'zh-CN') {
  const amount = toNumber(value)
  const abs = Math.abs(amount)
  if (locale === 'en-US') {
    if (abs >= 1_000_000_000) return `${(amount / 1_000_000_000).toFixed(2)}B`
    if (abs >= 1_000_000) return `${(amount / 1_000_000).toFixed(2)}M`
    if (abs >= 10_000) return `${(amount / 1_000).toFixed(1)}K`
    return formatYuan(amount)
  }
  if (abs >= 100_000_000) {
    return `${(amount / 100_000_000).toFixed(2)}亿`
  }
  if (abs >= 10_000) {
    return `${(amount / 10_000).toFixed(2)}万`
  }
  return formatYuan(amount)
}

/** 按开关输出金额或掩码。 */
export function displayAmount(value, visible = true) {
  return visible ? formatYuan(value) : '••••••'
}

const dayNumber = (value) => String(Number(String(value ?? '').split('-')[2]) || '')

function aggregateCategories(data) {
  const totals = new Map()
  const rows = data?.categories_source?.expense
  if (Array.isArray(rows)) {
    for (const row of rows) {
      const name = row?.['交易分类']
      if (!name) continue
      totals.set(name, (totals.get(name) ?? 0) + toNumber(row?.['金额']))
    }
  } else {
    const names = data?.categories?.expense?.names ?? []
    const amounts = data?.categories?.expense?.amounts ?? []
    names.forEach((name, index) => {
      totals.set(name, (totals.get(name) ?? 0) + toNumber(amounts[index]))
    })
  }

  const total = [...totals.values()].reduce((sum, value) => sum + value, 0)
  return [...totals.entries()]
    .map(([name, amount]) => ({
      name,
      amount,
      ratio: total > 0 ? Math.round((amount / total) * 1000) / 10 : 0
    }))
    .sort((a, b) => b.amount - a.amount)
}

function computeAsOfLabel(data, period, year, month, today, copy) {
  if (period !== 'month' || !(today instanceof Date) || Number.isNaN(today.getTime())) {
    return ''
  }
  if (Number(year) !== today.getFullYear() || Number(month) !== today.getMonth() + 1) {
    return ''
  }

  const expenses = data?.daily_data?.expense?.amounts ?? []
  const incomes = data?.daily_data?.income?.amounts ?? []
  const days = Math.max(expenses.length, incomes.length)
  let lastActive = 0
  for (let index = 0; index < days; index += 1) {
    if (toNumber(expenses[index]) > 0 || toNumber(incomes[index]) > 0) {
      lastActive = index + 1
    }
  }
  if (lastActive === 0 || lastActive >= days) return ''
  return copy.asOf(lastActive)
}

function buildChart(data, period) {
  const items =
    period === 'month'
      ? (data?.daily_data?.expense?.dates ?? []).map((date, index) => ({
          label: dayNumber(date),
          value: toNumber(data?.daily_data?.expense?.amounts?.[index])
        }))
      : (data?.trends?.months ?? []).map((month, index) => ({
          label: String(Number(String(month).split('-')[1]) || ''),
          value: toNumber(data?.trends?.expenses?.[index])
        }))

  const peak = items.reduce(
    (current, item) => (item.value > (current?.value ?? 0) ? item : current),
    null
  )

  return {
    items,
    peak,
    labelEvery: period === 'month' ? 5 : 1
  }
}

/**
 * 把年度 / 月度分析响应转换成海报可直接绘制的数据模型。
 *
 * @param {object} data 接口返回数据
 * @param {{ period?: 'year' | 'month', year?: number|string, month?: number|string }} options
 */
export function buildPosterModel(data, options = {}) {
  const period = options.period === 'month' ? 'month' : 'year'
  const year = String(options.year ?? '')
  const month = period === 'month' ? String(Number(options.month) || '') : ''

  const stats =
    period === 'month' ? data?.stats ?? {} : data?.yearly_stats ?? {}

  const chart = buildChart(data, period)
  const expense = toNumber(stats.total_expense)
  const income = toNumber(stats.total_income)
  const balance =
    stats.balance === undefined || stats.balance === null
      ? income - expense
      : toNumber(stats.balance)

  const activeDays =
    period === 'month'
      ? chart.items.filter((item) => item.value > 0).length
      : toNumber(stats.active_days)

  const totalCount =
    period === 'month'
      ? toNumber(stats.expense_count) + toNumber(stats.income_count)
      : toNumber(stats.total_count)

  const avgDailyExpense =
    period === 'month'
      ? (chart.items.length > 0 ? expense / chart.items.length : 0)
      : toNumber(stats.avg_daily_expense)

  const locale = options.locale === 'en-US' ? 'en-US' : 'zh-CN'
  const copy = posterCopy(locale)

  return {
    period,
    year,
    month,
    locale,
    asOfLabel: computeAsOfLabel(
      data,
      period,
      year,
      month,
      options.today ?? new Date(),
      copy
    ),
    appName: copy.appName,
    titleSuffix: period === 'month' ? copy.titleMonth : copy.titleYear,
    periodLabel: copy.periodLabel(year, month, period),
    subtitle: copy.subtitle,
    heroLabel: period === 'month' ? copy.heroMonth : copy.heroYear,
    incomeLabel: period === 'month' ? copy.incomeMonth : copy.incomeYear,
    balanceLabel: period === 'month' ? copy.balanceMonth : copy.balanceYear,
    chartTitle: period === 'month' ? copy.chartMonth : copy.chartYear,
    topLabel: copy.top,
    sortLabel: copy.sort,
    qrHint: copy.qrHint,
    qrPlaceholder: copy.qrPlaceholder,
    categoriesEmpty: copy.categoriesEmpty,
    chartEmpty: copy.chartEmpty,
    productName: copy.product,
    productSlogan: copy.slogan,
    demoSuffix: copy.demoSuffix,
    transactionsLabel: copy.transactions,
    countText: copy.countText,
    dailyAvgText: copy.dailyAvg,
    dailyAvgHidden: copy.dailyAvgHidden,
    peakText: (peak, value) => copy.peakText(peak.label, value, period),
    chart,
    expense,
    income,
    balance,
    totalCount,
    activeDays,
    avgDailyExpense,
    topCategories: aggregateCategories(data)
      .slice(0, 5)
      .map((item) => ({ ...item, name: localizeCategory(item.name, locale) }))
  }
}

// ==================== Canvas 绘制 ====================

const font = (weight, size) => `${weight} ${size}px ${FONT_STACK}`

export function getPosterTheme(id) {
  return POSTER_THEMES.find((theme) => theme.id === id) ?? POSTER_THEMES[0]
}

function roundRect(ctx, x, y, width, height, radius) {
  const r = Math.min(radius, width / 2, height / 2)
  ctx.beginPath()
  ctx.moveTo(x + r, y)
  ctx.arcTo(x + width, y, x + width, y + height, r)
  ctx.arcTo(x + width, y + height, x, y + height, r)
  ctx.arcTo(x, y + height, x, y, r)
  ctx.arcTo(x, y, x + width, y, r)
  ctx.closePath()
}

function fitFont(ctx, text, weight, size, maxWidth) {
  let current = size
  ctx.font = font(weight, current)
  while (ctx.measureText(text).width > maxWidth && current > 24) {
    current -= 2
    ctx.font = font(weight, current)
  }
  return current
}

function drawLogo(ctx, logo, theme) {
  const x = 112
  const y = 112
  const size = 96
  ctx.save()
  ctx.beginPath()
  ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  ctx.closePath()
  ctx.fillStyle = theme.pillBg
  ctx.fill()
  ctx.clip()
  if (logo && logo.complete && logo.naturalWidth > 0) {
    ctx.drawImage(logo, x, y, size, size)
  } else {
    ctx.fillStyle = theme.brand
    ctx.font = font(600, 40)
    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillText('遥', x + size / 2, y + size / 2 + 2)
  }
  ctx.restore()
  ctx.beginPath()
  ctx.arc(x + size / 2, y + size / 2, size / 2, 0, Math.PI * 2)
  ctx.lineWidth = 3
  ctx.strokeStyle = theme.line
  ctx.stroke()
}

function drawHeader(ctx, model, theme, options) {
  drawLogo(ctx, options.logo, theme)

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 34)
  ctx.fillText(`${model.appName} · ${model.titleSuffix}`, 236, 152)

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 24)
  ctx.fillText(model.subtitle, 236, 192)

  if (model.asOfLabel) {
    ctx.textAlign = 'right'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 20)
    ctx.fillText(model.asOfLabel, 968, 194)
  }

  const pill = `${model.periodLabel}${options.isDemo ? model.demoSuffix : ''}`
  ctx.font = font(500, 24)
  const pillWidth = Math.max(112, ctx.measureText(pill).width + 48)
  const pillX = 968 - pillWidth
  roundRect(ctx, pillX, 116, pillWidth, 52, 26)
  ctx.fillStyle = theme.pillBg
  ctx.fill()
  ctx.fillStyle = theme.pillInk
  ctx.textAlign = 'center'
  ctx.textBaseline = 'middle'
  ctx.fillText(pill, pillX + pillWidth / 2, 143)
}

function drawHero(ctx, model, theme, options) {
  const visibility = options.visibility
  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 26)
  ctx.fillText(model.heroLabel, 112, 306)

  const amountText = displayAmount(model.expense, visibility.expense)
  const amountFontSize = fitFont(ctx, `¥${amountText}`, 700, 96, 780)
  ctx.fillStyle = theme.ink
  ctx.font = font(700, amountFontSize)
  ctx.fillText('¥', 112, 408)
  const symbolWidth = ctx.measureText('¥').width
  ctx.fillText(amountText, 112 + symbolWidth + 6, 408)

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 24)
  ctx.fillText(
    visibility.expense
      ? model.dailyAvgText(formatYuan(model.avgDailyExpense))
      : model.dailyAvgHidden,
    112,
    448
  )

  const cards = [
    {
      label: model.incomeLabel,
      value: visibility.income ? `¥${formatYuan(model.income)}` : '••••',
      color: theme.ink
    },
    {
      label: model.balanceLabel,
      value: visibility.balance
        ? `${model.balance < 0 ? '-' : ''}¥${formatYuan(Math.abs(model.balance))}`
        : '••••',
      color: model.balance >= 0 ? theme.green : theme.red
    },
    {
      label: model.transactionsLabel,
      value: model.countText(model.totalCount),
      color: theme.ink
    }
  ]

  cards.forEach((card, index) => {
    const x = 112 + index * 292
    roundRect(ctx, x, 476, 272, 104, 24)
    ctx.fillStyle = theme.panel
    ctx.fill()

    ctx.textAlign = 'left'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 22)
    ctx.fillText(card.label, x + 24, 514)

    ctx.fillStyle = card.color
    ctx.font = font(600, 30)
    ctx.fillText(card.value, x + 24, 554)
  })
}

function drawChart(ctx, model, theme, options) {
  const visibility = options.visibility
  const x = 112
  const y = 608
  const width = 856
  const height = 296

  roundRect(ctx, x, y, width, height, 28)
  ctx.fillStyle = theme.panel
  ctx.fill()

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 28)
  ctx.fillText(model.chartTitle, x + 32, y + 52)

  ctx.textAlign = 'right'
  ctx.fillStyle = theme.muted
  ctx.font = font(400, 22)
  const peak = model.chart.peak
  const peakText = peak
    ? model.peakText(
        peak,
        visibility.peak ? formatCompactYuan(peak.value, model.locale) : ''
      )
    : ''
  if (peakText) ctx.fillText(peakText, x + width - 32, y + 52)

  const items = model.chart.items
  if (items.length === 0) {
    ctx.textAlign = 'center'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 24)
    ctx.fillText(model.chartEmpty, x + width / 2, y + height / 2)
    return
  }

  const max = Math.max(...items.map((item) => item.value), 0)
  const chartLeft = x + 40
  const chartRight = x + width - 40
  const baseline = y + 232
  const maxBarHeight = 122
  const slot = (chartRight - chartLeft) / items.length
  const barWidth = Math.min(40, slot * 0.56)

  items.forEach((item, index) => {
    const center = chartLeft + slot * index + slot / 2
    const ratio = max > 0 ? item.value / max : 0
    const barHeight = Math.max(6, maxBarHeight * ratio)
    const isPeak = max > 0 && item.value === max

    roundRect(ctx, center - barWidth / 2, baseline - barHeight, barWidth, barHeight, barWidth / 2)
    ctx.fillStyle = isPeak ? theme.brand : theme.bar
    ctx.fill()

    const showLabel =
      (index + 1) % model.chart.labelEvery === 0 || index === 0
    if (showLabel) {
      ctx.textAlign = 'center'
      ctx.fillStyle = isPeak ? theme.brand : theme.muted
      ctx.font = font(isPeak ? 600 : 400, 18)
      ctx.fillText(item.label, center, baseline + 30)
    }
  })
}

function drawCategories(ctx, model, theme, options) {
  const visibility = options.visibility
  const x = 112
  const titleY = 972

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 28)
  ctx.fillText(model.topLabel, x, titleY)

  ctx.textAlign = 'right'
  ctx.fillStyle = theme.muted
  ctx.font = font(400, 22)
  ctx.fillText(model.sortLabel, 968, titleY)

  if (model.topCategories.length === 0) {
    ctx.textAlign = 'left'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 24)
    ctx.fillText(model.categoriesEmpty, x, titleY + 64)
    return
  }

  const maxAmount = Math.max(...model.topCategories.map((item) => item.amount), 0)

  model.topCategories.forEach((item, index) => {
    const rowY = titleY + 44 + index * 48
    const centerY = rowY + 10
    const color = theme.palette[index % theme.palette.length]

    ctx.beginPath()
    ctx.arc(x + 14, centerY - 6, 14, 0, Math.PI * 2)
    ctx.fillStyle = color
    ctx.fill()

    ctx.textAlign = 'center'
    ctx.textBaseline = 'middle'
    ctx.fillStyle = '#FFFFFF'
    ctx.font = font(600, 18)
    ctx.fillText(String(index + 1), x + 14, centerY - 5)

    ctx.textAlign = 'left'
    ctx.textBaseline = 'alphabetic'
    ctx.font = font(600, 26)
    const nameWidth = ctx.measureText(item.name).width
    ctx.fillStyle = theme.ink
    ctx.fillText(item.name, x + 40, rowY + 4)

    ctx.fillStyle = theme.muted
    ctx.font = font(400, 21)
    ctx.fillText(`${item.ratio.toFixed(1)}%`, x + 40 + nameWidth + 14, rowY + 4)

    const trackX = x + 40
    const trackWidth = 620
    roundRect(ctx, trackX, rowY + 14, trackWidth, 7, 4)
    ctx.fillStyle = theme.track
    ctx.fill()

    const fillWidth = maxAmount > 0 ? Math.max(10, (item.amount / maxAmount) * trackWidth) : 10
    roundRect(ctx, trackX, rowY + 14, fillWidth, 7, 4)
    ctx.fillStyle = color
    ctx.fill()

    ctx.textAlign = 'right'
    ctx.fillStyle = theme.ink
    ctx.font = font(600, 26)
    ctx.fillText(
      visibility.categories ? formatCompactYuan(item.amount, model.locale) : '••••',
      968,
      rowY + 4
    )
  })
}

function drawFooter(ctx, model, theme, assets) {
  ctx.beginPath()
  ctx.moveTo(112, 1256)
  ctx.lineTo(968, 1256)
  ctx.strokeStyle = theme.line
  ctx.lineWidth = 2
  ctx.stroke()

  ctx.textAlign = 'left'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.ink
  ctx.font = font(600, 30)
  ctx.fillText(model.productName, 112, 1306)

  ctx.fillStyle = theme.muted
  ctx.font = font(400, 22)
  ctx.fillText(model.productSlogan, 112, 1338)

  ctx.fillStyle = theme.brand
  ctx.font = font(400, 19)
  ctx.fillText(PRODUCT_SITE_URL.replace('https://', ''), 112, 1370)

  const qrSize = 96
  const qrX = 856
  const qrY = 1264

  if (assets.qr && assets.qr.complete && assets.qr.naturalWidth > 0) {
    ctx.drawImage(assets.qr, qrX, qrY, qrSize, qrSize)
  } else {
    roundRect(ctx, qrX, qrY, qrSize, qrSize, 12)
    ctx.fillStyle = theme.panel
    ctx.fill()
    ctx.textAlign = 'center'
    ctx.fillStyle = theme.muted
    ctx.font = font(400, 16)
    ctx.textBaseline = 'middle'
    ctx.fillText(model.qrPlaceholder, qrX + qrSize / 2, qrY + qrSize / 2)
  }

  ctx.textAlign = 'center'
  ctx.textBaseline = 'alphabetic'
  ctx.fillStyle = theme.muted
  ctx.font = font(400, 18)
  ctx.fillText(model.qrHint, qrX + qrSize / 2, 1380)
}

/**
 * 把海报绘制到 1080×1440 的 canvas。
 *
 * @param {CanvasRenderingContext2D} ctx
 * @param {ReturnType<typeof buildPosterModel>} model
 * @param {{ logo?: HTMLImageElement, qr?: HTMLImageElement }} assets
 * @param {{
 *   theme?: string,
 *   visibility?: Record<string, boolean>,
 *   isDemo?: boolean
 * }} options
 */
export function drawSharePoster(ctx, model, assets = {}, options = {}) {
  const theme = getPosterTheme(options.theme)
  const visibility = { ...DEFAULT_AMOUNT_VISIBILITY, ...(options.visibility ?? {}) }
  const resolved = { visibility, isDemo: Boolean(options.isDemo), logo: assets.logo }

  ctx.save()
  ctx.clearRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)

  const background = ctx.createLinearGradient(0, 0, POSTER_WIDTH, POSTER_HEIGHT)
  background.addColorStop(0, theme.page[0])
  background.addColorStop(0.55, theme.page[1])
  background.addColorStop(1, theme.page[2])
  ctx.fillStyle = background
  ctx.fillRect(0, 0, POSTER_WIDTH, POSTER_HEIGHT)

  const glow = ctx.createRadialGradient(960, 60, 20, 960, 60, 420)
  glow.addColorStop(0, theme.glow1)
  glow.addColorStop(1, 'rgba(0, 0, 0, 0)')
  ctx.fillStyle = glow
  ctx.fillRect(520, 0, 560, 520)

  const glow2 = ctx.createRadialGradient(80, 1380, 20, 80, 1380, 380)
  glow2.addColorStop(0, theme.glow2)
  glow2.addColorStop(1, 'rgba(0, 0, 0, 0)')
  ctx.fillStyle = glow2
  ctx.fillRect(0, 980, 460, 460)

  ctx.shadowColor = theme.shadow
  ctx.shadowBlur = 48
  ctx.shadowOffsetY = 18
  roundRect(ctx, 56, 56, 968, 1328, 48)
  ctx.fillStyle = theme.card
  ctx.fill()
  ctx.shadowColor = 'transparent'
  ctx.shadowBlur = 0
  ctx.shadowOffsetY = 0

  drawHeader(ctx, model, theme, resolved)
  drawHero(ctx, model, theme, resolved)
  drawChart(ctx, model, theme, resolved)
  drawCategories(ctx, model, theme, resolved)
  drawFooter(ctx, model, theme, assets)
  ctx.restore()
}
