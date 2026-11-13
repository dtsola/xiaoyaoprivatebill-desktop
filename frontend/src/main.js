import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import router from './router'
import api from './api/client'
import i18n, { resolveLocale, setLocale } from './i18n'
import './assets/styles/main.css'

const app = createApp(App)

app.use(createPinia())
app.use(i18n)
app.use(router)

/**
 * 启动时先读取语言偏好，再挂载应用，避免首屏语言闪烁。
 * 没有 Tauri 环境（纯 Vite 调试）时按系统语言处理。
 */
async function bootstrap() {
  let preference = 'system'
  try {
    const settings = await api.desktopSettings()
    preference = settings?.language ?? 'system'
  } catch (error) {
    console.warn('[i18n] 读取语言偏好失败，按系统语言处理:', error)
  }

  const locale = resolveLocale(preference)
  setLocale(locale)

  try {
    await api.applyShellLanguage(locale)
  } catch (error) {
    console.warn('[i18n] 同步托盘/窗口语言失败:', error)
  }

  app.mount('#app')
}

bootstrap()
