<template>
  <div class="share-poster-page">
    <div class="page-header">
      <h1 class="page-title">分享海报</h1>
      <div v-if="availableYears.length" class="control-group">
        <button class="control-button" :disabled="!canGoPrev" aria-label="上一年" @click="prevYear">
          <ChevronLeft :size="'1em'" />
        </button>
        <span class="control-display">{{ currentYear }}年</span>
        <button class="control-button" :disabled="!canGoNext" aria-label="下一年" @click="nextYear">
          <ChevronRight :size="'1em'" />
        </button>
      </div>
      <div></div>
    </div>

    <div v-if="loading && !yearlyData" class="page-loading">
      <div class="loading-spinner"></div>
      <p>正在整理这一年的账单...</p>
    </div>

    <div v-else-if="yearlyData" class="share-poster-content">
      <SharePoster
        inline
        :data="yearlyData"
        :year="currentYear"
        :is-demo="sessionStore.isDemo"
      />
    </div>

    <div v-else class="empty-state">
      <ImageOff class="empty-icon" :size="'1em'" />
      <p>还没有可生成海报的账单数据</p>
      <div class="empty-actions">
        <button class="btn btn-primary" @click="$router.push('/settings')">上传账单</button>
        <button class="btn" @click="enterDemoMode">查看示例数据</button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { computed, onMounted, ref } from 'vue'
import { ChevronLeft, ChevronRight, ImageOff } from 'lucide-vue-next'

import SharePoster from '@/components/SharePoster.vue'
import { useDataStore } from '@/stores/data'
import { useFilterStore } from '@/stores/filter'
import { useSessionStore } from '@/stores/session'
import { useUiStore } from '@/stores/ui'

const dataStore = useDataStore()
const filterStore = useFilterStore()
const sessionStore = useSessionStore()
const uiStore = useUiStore()

const currentYear = ref(null)
const yearlyData = ref(null)
const loading = ref(false)

const availableYears = computed(() => dataStore.availableYears || [])

const canGoPrev = computed(() => {
  const index = availableYears.value.indexOf(parseInt(currentYear.value))
  return index >= 0 && index < availableYears.value.length - 1
})

const canGoNext = computed(() => {
  const index = availableYears.value.indexOf(parseInt(currentYear.value))
  return index > 0
})

async function loadData(year) {
  if (!year) return
  currentYear.value = year
  loading.value = true
  try {
    yearlyData.value = await dataStore.loadYearlyData({
      year,
      ...filterStore.getFilterParams()
    })
  } catch (error) {
    console.error('[SharePosterPage] 加载年度数据失败:', error)
    uiStore.showError(`加载数据失败: ${error.message}`)
  } finally {
    loading.value = false
  }
}

function prevYear() {
  const index = availableYears.value.indexOf(parseInt(currentYear.value))
  if (index < availableYears.value.length - 1) {
    loadData(availableYears.value[index + 1])
  }
}

function nextYear() {
  const index = availableYears.value.indexOf(parseInt(currentYear.value))
  if (index > 0) {
    loadData(availableYears.value[index - 1])
  }
}

async function enterDemoMode() {
  try {
    await sessionStore.enterDemoMode()
    uiStore.showSuccess('已进入演示模式')
    await dataStore.loadAvailableYears()
    const latest = dataStore.availableYears?.[0]
    if (latest) await loadData(latest)
  } catch (error) {
    console.error('[SharePosterPage] 进入演示模式失败:', error)
    uiStore.showError(`进入演示模式失败: ${error.message}`)
  }
}

onMounted(async () => {
  await dataStore.loadAvailableYears()
  const latest = dataStore.availableYears?.[0]
  if (latest) {
    await loadData(latest)
  }
})
</script>

<style scoped>
.share-poster-page {
  padding-bottom: 24px;
}

.share-poster-content {
  padding: 0 20px;
}

.page-loading {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 80px 0;
  color: var(--secondary-text, #6e6e73);
}

.loading-spinner {
  width: 28px;
  height: 28px;
  border: 3px solid rgba(0, 122, 255, 0.2);
  border-top-color: var(--primary-color, #007aff);
  border-radius: 50%;
  animation: share-spin 0.8s linear infinite;
}

@keyframes share-spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 80px 0;
  color: var(--secondary-text, #6e6e73);
}

.empty-icon {
  font-size: 48px;
  color: var(--secondary-text, #6e6e73);
}

.empty-actions {
  display: flex;
  gap: 12px;
}
</style>
