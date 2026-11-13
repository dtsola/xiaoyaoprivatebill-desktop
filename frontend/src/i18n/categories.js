/**
 * 后端账单分类名为中文，英文界面按固定词典映射。
 * 未知或用户自定义分类保留原文，避免信息丢失。
 */
export const CATEGORY_LABELS_EN = {
  餐饮美食: 'Dining',
  酒店旅游: 'Travel',
  交通出行: 'Transport',
  服饰装扮: 'Clothing',
  日用百货: 'Household',
  数码电器: 'Electronics',
  生活服务: 'Services',
  爱车养车: 'Car care',
  运动户外: 'Sports & outdoors',
  文化休闲: 'Leisure',
  教育培训: 'Education',
  家居家装: 'Home & furniture',
  住房物业: 'Housing',
  医疗健康: 'Health',
  充值缴费: 'Top-ups & bills',
  公共服务: 'Public services',
  商业服务: 'Business services',
  信用借还: 'Credit & loans',
  母婴亲子: 'Family & kids',
  收入: 'Income',
  转账红包: 'Transfers & red packets',
  保险: 'Insurance',
  其他: 'Other',
  商户消费: 'Merchant payments',
  扫二维码付款: 'QR payments',
  转账: 'Transfers',
  微信红包: 'WeChat red packet',
  '微信红包（单发）': 'WeChat red packet (single)',
  '微信红包（群红包）': 'WeChat red packet (group)',
  群收款: 'Group collection',
  亲子: 'Family & kids',
  美容美发: 'Beauty & hair',
  宠物: 'Pets',
  宠物用品: 'Pet supplies',
  理财: 'Investing',
  购物: 'Shopping',
  娱乐: 'Entertainment',
  通讯: 'Telecom',
  住房: 'Housing',
  教育: 'Education',
  旅行: 'Travel',
  餐饮: 'Dining',
  支付宝: 'Alipay',
  微信: 'WeChat',
  周一: 'Mon',
  周二: 'Tue',
  周三: 'Wed',
  周四: 'Thu',
  周五: 'Fri',
  周六: 'Sat',
  周日: 'Sun',
  工作日: 'Weekday',
  周末: 'Weekend'
}

/**
 * 按语言输出分类显示名。
 *
 * @param {string} name 后端原始分类名
 * @param {string} locale 当前语言
 * @returns {string} 用于展示的分类名
 */
export function localizeCategory(name, locale = 'zh-CN') {
  if (locale !== 'en-US') return name
  return CATEGORY_LABELS_EN[name] ?? name
}

/**
 * 金额区间显示名：后端为「0-100元」「>5000元」，英文输出「0–100 CNY」「> 5,000 CNY」。
 *
 * @param {string} name 后端原始区间名
 * @param {string} locale 当前语言
 * @returns {string} 用于展示的区间名
 */
export function localizeAmountRange(name, locale = 'zh-CN') {
  if (locale !== 'en-US' || typeof name !== 'string') return name
  const number = (value) => Number(value.replace(/,/g, '')).toLocaleString('en-US')
  return name
    .replace(/([0-9][0-9,.]*)\s*[-~]\s*([0-9][0-9,.]*)\s*元/g, (_, start, end) => `${number(start)}–${number(end)} CNY`)
    .replace(/(>=|>|≤|<)\s*([0-9][0-9,.]*)\s*元/g, (_, operator, value) => `${operator} ${number(value)} CNY`)
    .replace(/([0-9][0-9,.]*)\s*元/g, (_, value) => `${number(value)} CNY`)
}

/**
 * 消费场景取值：渠道 / 时段 / 金额层级 / 支付方式。
 * 后端固定输出中文，英文界面按此词典转换。
 */
export const SCENARIO_LABELS_EN = {
  线上: 'Online',
  线下: 'Offline',
  '清晨(6-9点)': 'Early morning (6–9)',
  '上午(9-12点)': 'Morning (9–12)',
  '中午(12-14点)': 'Noon (12–14)',
  '下午(14-17点)': 'Afternoon (14–17)',
  '傍晚(17-20点)': 'Evening (17–20)',
  '晚上(20-23点)': 'Night (20–23)',
  '深夜(23-6点)': 'Late night (23–6)',
  '大额(1000+)': 'Large (1000+)',
  '中额(300-1000)': 'Medium (300–1000)',
  '小额(100-300)': 'Small (100–300)',
  '零花(0-100)': 'Pocket money (0–100)',
  支付宝: 'Alipay',
  微信: 'WeChat'
}

/**
 * 按语言输出消费场景名称。
 *
 * @param {string} name 后端原始场景名
 * @param {string} locale 当前语言
 * @returns {string} 用于展示的场景名
 */
export function localizeScenarioName(name, locale = 'zh-CN') {
  if (locale !== 'en-US') return name
  return SCENARIO_LABELS_EN[name] ?? localizeCategory(name, locale)
}
