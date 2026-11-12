<template>
  <div class="share-poster-page">
    <div class="page-header">
      <h1 class="page-title">分享海报</h1>
      <div class="share-filters">
        <div class="period-tabs" role="tablist" aria-label="账单范围">
          <button
            type="button"
            class="period-tab"
            :class="{ active: period === 'year' }"
            role="tab"
            :aria-selected="period === 'year'"
            @click="setPeriod('year')"
          >
            年度账单
          </button>
          <button
            type="button"
            class="period-tab"
            :class="{ active: period === 'month' }"
            role="tab"
            :aria-selected="period === 'month'"
            @click="setPeriod('month')"
          >
            月度账单
          </button>
        </div>

        <select
          v-model.number="selectedYear"
          class="filter-select"
          aria-label="选择年份"
          @change="handleYearChange"
        >
          <option v-for="year in availableYears" :key="year" :value="year">
            {{ year }} 年
          </option>
        </select>

        <select
          v-if="period === 'month'"
          v-model="selectedMonth"
          class="filter-select"
          aria-label="选择月份"
          @change="loadData"
        >
          <option v-for="month in monthOptions" :key="month.value" :value="month.value">
            {{ month.label }}
          </option>
        </select>
      </div>
      <div></div>
    </div>

    <div v-if="loading && !posterData" class="page-loading">
      <div class="loading-spinner"></div>
      <p>正在整理账单...</p>
    </div>

    <div v-else-if="posterData" class="share-poster-content">
      <SharePoster
        :data="posterData"
        :year="posterYear"
        :period="period"
        :month="posterMonth"
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
import { ImageOff } from 'lucide-vue-next'

import SharePoster from '@/components/SharePoster.vue'
import { useDataStore } from '@/stores/data'
import { useFilterStore } from '@/stores/filter'
import { useSessionStore } from '@/stores/session'
import { useUiStore } from '@/stores/ui'

const dataStore = useDataStore()
const filterStore = useFilterStore()
const sessionStore = useSessionStore()
const uiStore = useUiStore()

const period = ref('year')
const selectedYear = ref(null)
const selectedMonth = ref('')
const posterData = ref(null)
const loading = ref(false)

const availableYears = computed(() => dataStore.availableYears || [])

const allMonths = computed(() => dataStore.availableDates?.months || [])

const monthOptions = computed(() =>
  allMonths.value
    .filter((month) => String(month).startsWith(`${selectedYear.value}-`))
    .map((month) => ({
      value: month,
      label: `${Number(String(month).split('-')[1])} 月`
    }))
)

const posterYear = computed(() =>
  period.value === 'month' ? Number(String(selectedMonth.value).split('-')[0]) : selectedYear.value
)

const posterMonth = computed(() =>
  period.value === 'month' ? Number(String(selectedMonth.value).split('-')[1]) : ''
)

function firstMonthOfYear(year) {
  return allMonths.value.find((month) => String(month).startsWith(`${year}-`)) ?? ''
}

async function loadData() {
  if (!selectedYear.value) return
  if (period.value === 'month' && !selectedMonth.value) {
    posterData.value = null
    return
  }

  loading.value = true
  try {
    const filters = filterStore.getFilterParams()
    posterData.value =
      period.value === 'year'
        ? await dataStore.loadYearlyData({ year: selectedYear.value, ...filters })
        : await dataStore.loadMonthlyData({
            year: posterYear.value,
            month: posterMonth.value,
            ...filters
          })
  } catch (error) {
    console.error('[SharePosterPage] 加载账单数据失败:', error)
    uiStore.showError(`加载数据失败: ${error.message}`)
  } finally {
    loading.value = false
  }
}

function setPeriod(next) {
  period.value = next
  if (next === 'month' && !selectedMonth.value) {
    selectedMonth.value = firstMonthOfYear(selectedYear.value)
  }
  loadData()
}

function handleYearChange() {
  if (period.value === 'month') {
    selectedMonth.value = firstMonthOfYear(selectedYear.value)
  }
  loadData()
}

async function enterDemoMode() {
  try {
    await sessionStore.enterDemoMode()
    uiStore.showSuccess('已进入演示模式')
    await Promise.all([dataStore.loadAvailableYears(), dataStore.loadAvailableDates()])
    selectedYear.value = dataStore.availableYears?.[0] ?? null
    selectedMonth.value = firstMonthOfYear(selectedYear.value)
    await loadData()
  } catch (error) {
    console.error('[SharePosterPage] 进入演示模式失败:', error)
    uiStore.showError(`进入演示模式失败: ${error.message}`)
  }
}

onMounted(async () => {
  await Promise.all([dataStore.loadAvailableYears(), dataStore.loadAvailableDates()])
  selectedYear.value = dataStore.availableYears?.[0] ?? null
  selectedMonth.value = firstMonthOfYear(selectedYear.value)
  await loadData()
})
</script>

<style scoped>
.share-poster-page {
  padding-bottom: 24px;
}

.share-filters {
  display: flex;
  align-items: center;
  gap: 12px;
}

.period-tabs {
  display: inline-flex;
  padding: 3px;
  border-radius: 12px;
  background: var(--hover-bg, #e9e9ee);
}

.period-tab {
  height: 32px;
  padding: 0 16px;
  border: none;
  border-radius: 9px;
  background: transparent;
  color: var(--secondary-text, #6e6e73);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
}

.period-tab.active {
  background: var(--card-bg, #fff);
  color: var(--primary-color, #007aff);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

.filter-select {
  height: 36px;
  padding: 0 30px 0 12px;
  border: 1px solid var(--border-color, #e5e5ea);
  border-radius: 10px;
  background: var(--card-bg, #fff);
  color: var(--text-color, #1c1c1e);
  font-size: 13px;
  cursor: pointer;
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

@media (max-width: 768px) {
  .page-header {
    grid-template-columns: 1fr;
    gap: 12px;
    text-align: center;
  }

  .share-filters {
    flex-wrap: wrap;
    justify-content: center;
  }
}
</style>
