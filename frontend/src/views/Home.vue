<template>
  <div class="welcome-section">
    <h1 class="welcome-title">{{ t('home.title') }}</h1>
    <p class="welcome-subtitle">
      {{ t('home.subtitle') }}<br />
      {{ t('home.privacySubtitle') }}
    </p>

    <div class="features-grid">
      <div class="feature-card">
        <div class="feature-icon feature-icon-analysis">
          <ChartPie :size="'1em'" />
        </div>
        <h3 class="feature-title">{{ t('home.features.analysisTitle') }}</h3>
        <p class="feature-description">
          {{ t('home.features.analysisDesc') }}
        </p>
      </div>
      <div class="feature-card">
        <div class="feature-icon feature-icon-privacy">
          <ShieldCheck :size="'1em'" />
        </div>
        <h3 class="feature-title">{{ t('home.features.privacyTitle') }}</h3>
        <p class="feature-description">
          {{ t('home.features.privacyDesc') }}
        </p>
      </div>
      <div class="feature-card">
        <div class="feature-icon feature-icon-trend">
          <ChartLine :size="'1em'" />
        </div>
        <h3 class="feature-title">{{ t('home.features.trendTitle') }}</h3>
        <p class="feature-description">
          {{ t('home.features.trendDesc') }}
        </p>
      </div>
      <div class="feature-card">
        <div class="feature-icon feature-icon-search">
          <Search :size="'1em'" />
        </div>
        <h3 class="feature-title">{{ t('home.features.searchTitle') }}</h3>
        <p class="feature-description">
          {{ t('home.features.searchDesc') }}
        </p>
      </div>
    </div>

    <div class="upload-section">
      <h2 class="upload-title">{{ t('home.uploadTitle') }}</h2>
      <p class="privacy-notice">
        {{ t('home.uploadHint') }}<br />
        {{ t('home.privacyHint') }}
      </p>
      <div class="action-buttons">
        <router-link to="/settings" class="start-button">
          <Upload :size="'1em'" />
          {{ t('home.uploadButton') }}
        </router-link>
        <button @click="enterDemoMode" class="demo-button">
          <Eye :size="'1em'" />
          {{ t('home.demoButton') }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ChartLine, ChartPie, Eye, Search, ShieldCheck, Upload } from 'lucide-vue-next'
import { useRouter } from 'vue-router'
import { useUiStore } from '@/stores/ui'
import { useI18n } from 'vue-i18n'
import { useSessionStore } from '@/stores/session'

const router = useRouter()
const uiStore = useUiStore()
const { t } = useI18n()
const sessionStore = useSessionStore()

async function enterDemoMode() {
  try {
    await sessionStore.enterDemoMode()
    uiStore.showSuccess(t('layout.demo.entered'))
    router.push('/yearly')
  } catch (error) {
    console.error('Error:', error)
    uiStore.showError(`${t('layout.demo.enterFailed')}: ${error.message}`)
  }
}
</script>

<style scoped>
/* 完全复制老前端的样式 */
.welcome-section {
  text-align: center;
  padding: 48px 24px;
  /* 随窗口自适应：窄窗口铺满内容区，宽窗口最多 1200px */
  width: 100%;
  max-width: 1200px;
  margin: 0 auto;
}

.welcome-title {
  font-size: 32px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 16px;
  letter-spacing: -0.025em;
}

.welcome-subtitle {
  font-size: 18px;
  color: var(--secondary-text);
  margin-bottom: 48px;
  line-height: 1.5;
}

.features-grid {
  display: grid;
  /* 默认两列；宽窗口改为四列，窄窗口单列 */
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 24px;
  margin-bottom: 48px;
}

.feature-card {
  background: var(--card-bg);
  border-radius: var(--radius-lg);
  padding: 24px;
  text-align: left;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);
  transition: all 0.3s ease;
}

.feature-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.feature-icon {
  width: 56px;
  height: 56px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-md);
  margin-bottom: 16px;
  font-size: 24px;
}

/* 为每个特色图标添加彩色背景 */
.feature-icon-analysis {
  background: rgba(0, 122, 255, 0.1);
  color: #007AFF;
}

.feature-icon-privacy {
  background: rgba(52, 199, 89, 0.1);
  color: #34C759;
}

.feature-icon-trend {
  background: rgba(255, 149, 0, 0.1);
  color: #FF9500;
}

.feature-icon-search {
  background: rgba(175, 82, 222, 0.1);
  color: #AF52DE;
}

.feature-title {
  font-size: 17px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 12px;
}

.feature-description {
  font-size: 14px;
  color: var(--secondary-text);
  line-height: 1.5;
  margin: 0;
}

.upload-section {
  background: var(--card-bg);
  border-radius: var(--radius-lg);
  padding: 32px;
  margin-bottom: 48px;
  text-align: center;
}

.upload-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 16px;
}

.privacy-notice {
  font-size: 14px;
  color: var(--secondary-text);
  max-width: 600px;
  margin: 0 auto;
  line-height: 1.5;
}

.action-buttons {
  display: flex;
  justify-content: center;
  gap: 16px;
  margin-top: 24px;
}

.start-button {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: var(--primary-color);
  color: white;
  border: none;
  border-radius: var(--radius-sm);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  text-decoration: none;
  transition: all 0.2s ease;
  min-width: 180px;
  height: 48px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.start-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.start-button i {
  font-size: 18px;
}

.demo-button {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: white;
  color: var(--primary-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  font-size: 16px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 180px;
  height: 48px;
  justify-content: center;
}

.demo-button:hover {
  background: var(--hover-bg);
  transform: translateY(-1px);
  box-shadow: var(--shadow-card);
}

/* 宽窗口（内容区 ≥ 1200px）四张卡片并排，避免两侧留白 */
@media (min-width: 1440px) {
  .features-grid {
    grid-template-columns: repeat(4, minmax(0, 1fr));
  }
}

@media (max-width: 768px) {
  .features-grid {
    grid-template-columns: minmax(0, 1fr);
  }

  .start-button {
    padding: 10px 20px;
    font-size: 14px;
    min-width: 120px;
  }
}
</style>
