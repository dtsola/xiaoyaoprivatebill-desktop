import { createI18n } from 'vue-i18n'

import enUS from './locales/en-US'
import zhCN from './locales/zh-CN'

export const SUPPORTED_LOCALES = ['zh-CN', 'en-US']
export const DEFAULT_LOCALE = 'zh-CN'

/**
 * 读取系统语言：中文系统用 zh-CN，其他语言用 en-US。
 * 无法判断时回退到中文。
 */
export function detectSystemLocale() {
  const language =
    typeof navigator !== 'undefined' && navigator.language ? navigator.language : ''
  if (!language) return DEFAULT_LOCALE
  return String(language).toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

/**
 * 解析语言偏好：`system` 或未知值都走系统语言。
 */
export function resolveLocale(preference) {
  return SUPPORTED_LOCALES.includes(preference) ? preference : detectSystemLocale()
}

export const i18n = createI18n({
  legacy: false,
  locale: DEFAULT_LOCALE,
  fallbackLocale: DEFAULT_LOCALE,
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS
  },
  missingWarn: false,
  fallbackWarn: false
})

/**
 * 切换界面语言，并同步 `<html lang>` 与文档标题。
 */
export function setLocale(preference) {
  const locale = resolveLocale(preference)
  i18n.global.locale.value = locale
  if (typeof document !== 'undefined') {
    document.documentElement.lang = locale
    document.title = i18n.global.t('layout.appNameFull')
  }
  return locale
}

export function currentLocale() {
  return i18n.global.locale.value
}

export default i18n
