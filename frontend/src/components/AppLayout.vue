<template>
  <div class="app-shell" :class="{ 'platform-mac': isMac, 'sidebar-collapsed': collapsed }">
    <!-- 顶部拖拽条：给没有页面标题区的页面（首页）提供拖拽与双击最大化；
         有标题区的页面，标题区本身固定在更上层并自带拖拽标记 -->
    <div class="drag-strip" data-tauri-drag-region @dblclick="toggleMaximize"></div>

    <!-- 窗口按钮：26px 圆角按钮，右上角 -->
    <div class="window-controls" v-if="!isMac">
      <button class="window-btn" :aria-label="t('layout.window.minimize')" :title="t('layout.window.minimize')" @click="minimizeWindow">
        <Minus :size="14" />
      </button>
      <button
        class="window-btn"
        :aria-label="isMaximized ? t('layout.window.restore') : t('layout.window.maximize')"
        :title="isMaximized ? t('layout.window.restore') : t('layout.window.maximize')"
        @click="toggleMaximize"
      >
        <Copy v-if="isMaximized" :size="12" />
        <Square v-else :size="11" />
      </button>
      <button class="window-btn window-close" :aria-label="t('layout.window.close')" :title="t('layout.window.close')" @click="closeWindow">
        <X :size="14" />
      </button>
    </div>

    <!-- 演示模式横幅 -->
    <div v-if="sessionStore.isDemo" class="demo-banner">
      <div class="demo-content">
        <FlaskConical :size="16" />
        <span>{{ t('layout.demo.banner') }}</span>
      </div>
      <button @click="exitDemoMode" class="exit-demo-btn">{{ t('layout.demo.exit') }}</button>
    </div>

    <div class="app-container">
      <!-- 侧边栏：整列贴到窗口顶部，LOGO 区域可拖拽窗口 -->
      <aside class="sidebar">
        <div class="logo" data-tauri-drag-region @dblclick="toggleMaximize">
          <img
            src="/images/logo_128.png"
            :alt="t('layout.logoAlt')"
            class="logo-icon"
            data-tauri-drag-region
          />
          <span data-tauri-drag-region>{{ t('layout.appName') }}</span>
          <button
            class="sidebar-toggle"
            :aria-label="collapsed ? t('layout.sidebar.expand') : t('layout.sidebar.collapse')"
            :title="collapsed ? t('layout.sidebar.expand') : t('layout.sidebar.collapse')"
            @click="toggleSidebar"
          >
            <ChevronRight v-if="collapsed" :size="15" />
            <ChevronLeft v-else :size="15" />
          </button>
        </div>
        <nav class="nav-menu">
          <router-link to="/" class="nav-item" :class="{ active: $route.path === '/' }">
            <House class="nav-icon icon-home" :size="18" />
            <span>{{ t('nav.home') }}</span>
          </router-link>
          <router-link
            to="/share"
            class="nav-item nav-item-share"
            :title="t('layout.shareEntry')"
            :class="{ active: $route.path === '/share' }"
          >
            <Share2 class="nav-icon icon-share" :size="18" />
            <span>{{ t('nav.share') }}</span>
          </router-link>
          <router-link to="/yearly" class="nav-item" :class="{ active: $route.path === '/yearly' }">
            <CalendarDays class="nav-icon icon-yearly" :size="18" />
            <span>{{ t('nav.yearly') }}</span>
          </router-link>
          <router-link to="/monthly" class="nav-item" :class="{ active: $route.path === '/monthly' }">
            <ChartLine class="nav-icon icon-monthly" :size="18" />
            <span>{{ t('nav.monthly') }}</span>
          </router-link>
          <router-link to="/category" class="nav-item" :class="{ active: $route.path === '/category' }">
            <Tags class="nav-icon icon-category" :size="18" />
            <span>{{ t('nav.category') }}</span>
          </router-link>
          <router-link to="/time" class="nav-item" :class="{ active: $route.path === '/time' }">
            <Clock class="nav-icon icon-time" :size="18" />
            <span>{{ t('nav.time') }}</span>
          </router-link>
          <router-link to="/insights" class="nav-item" :class="{ active: $route.path === '/insights' }">
            <Lightbulb class="nav-icon icon-insights" :size="18" />
            <span>{{ t('nav.insights') }}</span>
          </router-link>
          <router-link to="/transactions" class="nav-item" :class="{ active: $route.path === '/transactions' }">
            <Receipt class="nav-icon icon-transactions" :size="18" />
            <span>{{ t('nav.transactions') }}</span>
          </router-link>
          <router-link to="/agent" class="nav-item" :class="{ active: $route.path === '/agent' }">
            <Bot class="nav-icon icon-agent" :size="18" />
            <span>{{ t('nav.agent') }}</span>
          </router-link>
          <button
            type="button"
            class="nav-item nav-item-external"
            :title="t('layout.openClawSite')"
            @click="openClawSite"
          >
            <PawPrint class="nav-icon icon-claw" :size="18" />
            <span>{{ t('nav.claw') }}</span>
            <ArrowUpRight class="nav-external" :size="13" />
          </button>
          <router-link to="/settings" class="nav-item" :class="{ active: $route.path === '/settings' }">
            <Settings class="nav-icon icon-settings" :size="18" />
            <span>{{ t('nav.settings') }}</span>
          </router-link>
          <router-link to="/about-author" class="nav-item" :class="{ active: $route.path === '/about-author' }">
            <CircleUserRound class="nav-icon icon-author" :size="18" />
            <span>{{ t('nav.author') }}</span>
          </router-link>
        </nav>
      </aside>

      <!-- 主内容区域 -->
      <main class="content">
        <router-view />
      </main>
    </div>
  </div>

  <!-- 筛选菜单 (独立于侧边栏) -->
  <div class="floating-menu" :class="{ collapsed }">
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'all' }"
      @click="setFilter('all')"
    >
      <List :size="16" />
      <span>{{ t('layout.filters.all') }}</span>
    </button>
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'large' }"
      @click="setFilter('large')"
    >
      <Coins :size="16" />
      <span>{{ t('layout.filters.large') }}</span>
    </button>
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'small' }"
      @click="setFilter('small')"
    >
      <Coffee :size="16" />
      <span>{{ t('layout.filters.small') }}</span>
    </button>
  </div>

  <!-- Toast 组件 -->
  <Toast v-if="uiStore.toast.show" />

  <!-- Modal 组件 -->
  <Modal v-if="uiStore.modal.show" />

  <!-- 全局加载状态 -->
  <div v-if="uiStore.globalLoading" class="global-loader">
    <div class="loader-content">
      <div class="spinner"></div>
      <div class="loader-text">{{ t('layout.globalLoading') }}</div>
    </div>
  </div>
</template>

<script setup>
import { computed, nextTick, onMounted, onUnmounted, ref, watch, watchEffect } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  ArrowUpRight,
  Bot,
  CalendarDays,
  ChartLine,
  ChevronLeft,
  ChevronRight,
  CircleUserRound,
  Clock,
  Coffee,
  Coins,
  Copy,
  FlaskConical,
  House,
  Lightbulb,
  List,
  Minus,
  PawPrint,
  Receipt,
  Settings,
  Share2,
  Square,
  Tags,
  X
} from 'lucide-vue-next'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useUiStore } from '@/stores/ui'
import { useSessionStore } from '@/stores/session'
import { useFilterStore } from '@/stores/filter'
import { useI18n } from 'vue-i18n'
import Toast from '@/components/common/Toast.vue'
import Modal from '@/components/common/Modal.vue'

const router = useRouter()
const route = useRoute()
const uiStore = useUiStore()
const sessionStore = useSessionStore()
const filterStore = useFilterStore()
const { t } = useI18n()

let unlistenCloseHint = null

// 窗口外观：Windows 由 Rust 去掉原生边框，这里渲染拖拽区与按钮；
// macOS 保留原生红绿灯，只需留出左侧空间。
const isMac = typeof navigator !== 'undefined' && /Mac/i.test(navigator.userAgent)
const isMaximized = ref(false)

// —— 侧边栏折叠：宽窗口听持久偏好，窄窗口自动折叠、可本次会话手动展开 ——
const SIDEBAR_STORAGE_KEY = 'xiaoyaobill.sidebar'
const SIDEBAR_AUTO_COLLAPSE_WIDTH = 1100
const preferred = ref(
  typeof localStorage !== 'undefined' &&
    localStorage.getItem(SIDEBAR_STORAGE_KEY) === 'collapsed'
    ? 'collapsed'
    : 'expanded'
)
const windowWidth = ref(typeof window !== 'undefined' ? window.innerWidth : 1280)
const sessionExpanded = ref(false)
const collapsed = computed(() =>
  windowWidth.value < SIDEBAR_AUTO_COLLAPSE_WIDTH
    ? !sessionExpanded.value
    : preferred.value === 'collapsed'
)

function toggleSidebar() {
  if (windowWidth.value < SIDEBAR_AUTO_COLLAPSE_WIDTH) {
    // 窄窗口只影响本次会话，不覆盖持久偏好
    sessionExpanded.value = !sessionExpanded.value
    return
  }
  preferred.value = collapsed.value ? 'expanded' : 'collapsed'
  localStorage.setItem(SIDEBAR_STORAGE_KEY, preferred.value)
}

function syncWindowWidth() {
  windowWidth.value = window.innerWidth
}

// 页面标题区已固定到窗口顶部：给标题区与标题文字打上拖拽标记，
// 空白处/标题处都能拖动窗口，按钮与筛选控件保持可点。
function decoratePageChrome() {
  document.querySelectorAll('.page-header, .category-page-header').forEach((header) => {
    header.setAttribute('data-tauri-drag-region', '')
    header.querySelectorAll('h1, h2, .page-title, .page-subtitle').forEach((el) => {
      el.setAttribute('data-tauri-drag-region', '')
    })
    if (!header.dataset.chromeBound) {
      header.dataset.chromeBound = '1'
      header.addEventListener('dblclick', toggleMaximize)
    }
  })
}

// 写到 <html data-sidebar>，供全局 CSS 变量（内容区/浮动菜单等）使用
watchEffect(() => {
  document.documentElement.dataset.sidebar = collapsed.value ? 'collapsed' : 'expanded'
})
let appWindow = null
let unlistenResize = null

function windowHandle() {
  if (!appWindow && typeof window !== 'undefined' && window.__TAURI_INTERNALS__) {
    appWindow = getCurrentWindow()
  }
  return appWindow
}

function minimizeWindow() {
  void windowHandle()
    ?.minimize()
    .catch(() => {})
}

async function openClawSite() {
  try {
    await openUrl('https://www.xiaoyaosai.com/xiaoyaoclaw/')
  } catch (error) {
    uiStore.showError(`${t('layout.openClawFailed')}: ${error.message}`)
  }
}

async function toggleMaximize() {
  const win = windowHandle()
  if (!win) return
  try {
    await win.toggleMaximize()
    isMaximized.value = await win.isMaximized()
  } catch {
    // 浏览器预览无窗口 API，静默忽略
  }
}

function closeWindow() {
  // 走 Rust 的 CloseRequested：按设置决定收进托盘还是退出。
  void windowHandle()
    ?.close()
    .catch(() => {})
}

async function syncMaximized() {
  const win = windowHandle()
  if (!win || isMac) return
  try {
    isMaximized.value = await win.isMaximized()
  } catch {
    isMaximized.value = false
  }
}

onMounted(async () => {
  window.addEventListener('resize', syncWindowWidth)
  await nextTick()
  decoratePageChrome()
  unlistenCloseHint = await listen('desktop://close-to-tray-hint', () => {
    uiStore.showSuccess(t('layout.closeToTray'))
  })

  // 最大化状态跟随系统（双击拖拽区、Win+↑、任务栏菜单都会触发 resize）
  const win = windowHandle()
  if (win && !isMac) {
    await syncMaximized()
    unlistenResize = await win.onResized(syncMaximized)
  }
})

onUnmounted(() => {
  window.removeEventListener('resize', syncWindowWidth)
  if (unlistenCloseHint) {
    unlistenCloseHint()
    unlistenCloseHint = null
  }
  if (unlistenResize) {
    unlistenResize()
    unlistenResize = null
  }
})

async function exitDemoMode() {
  if (confirm(t('layout.demo.confirmExit'))) {
    try {
      await sessionStore.exitDemoMode()
      uiStore.showSuccess(t('layout.demo.exited'))
      router.replace('/')
    } catch (error) {
      uiStore.showError(`${t('layout.demo.exitFailed')}: ${error.message}`)
    }
  }
}

function setFilter(filterType) {
  console.log('[AppLayout] Setting filter to:', filterType)
  filterStore.setFilter(filterType)
  console.log('[AppLayout] Filter is now:', filterStore.currentFilter)
}

onMounted(async () => {
  // 加载会话状态
  await sessionStore.loadSessionStatus()
})

// 路由切换后页面头部会重建，重新打拖拽标记
watch(
  () => route.path,
  async () => {
    await nextTick()
    decoratePageChrome()
  }
)
</script>

<style scoped>
/* 完全复制老前端的样式 */
/* 应用外壳：固定视口高度；顶部 36px 留给拖拽条与窗口按钮 */
/* 侧边栏当前宽度：展开 240 / 收起 64，写在 html 上供固定定位元素共用 */
:global(html) {
  --sb-current-width: var(--sidebar-width);
}

:global(html[data-sidebar="collapsed"]) {
  --sb-current-width: 64px;
}

.app-shell {
  position: relative;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  /* 顶部 56px 为统一标题区：页面标题与筛选控件都在这里 */
  padding-top: 56px;
}

/* 拖拽条只覆盖内容区：侧边栏占满窗口高度，LOGO 区也可拖拽 */
/* 顶部拖拽条：只在标题区下方生效（无标题区页面如首页用） */
.drag-strip {
  position: absolute;
  top: 0;
  left: var(--sb-current-width);
  right: 96px;
  height: 56px;
  z-index: 2000;
  user-select: none;
}

/* 窗口按钮：与 xiaoyaotime 对齐（26px、右上角、圆角悬停） */
.window-controls {
  position: absolute;
  top: 0;
  right: 0;
  height: 56px;
  display: flex;
  align-items: center;
  gap: 4px;
  padding-right: 10px;
  z-index: 3000;
}

.window-btn {
  width: 26px;
  height: 26px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--secondary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.window-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: var(--text-color);
}

.window-btn:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: -2px;
}

.window-close:hover {
  background: #E81123;
  color: #ffffff;
}

.app-container {
  flex: 1;
  min-height: 0;
  display: flex;
}

/* 侧边栏贴到窗口顶部，盖住拖拽条所在高度 */
.sidebar {
  position: absolute;
  top: 0;
  left: 0;
  bottom: 0;
  width: var(--sb-current-width);
  height: 100%;
  display: flex;
  flex-direction: column;
  transition: width 0.22s ease;
  /* 比卡片白底略灰，避免整块侧栏发白 */
  background: #F2F2F4;
  border-right: 1px solid var(--border-color);
  z-index: 2500;
  overflow-y: auto;
}

/* macOS：给左上角红绿灯留出空间 */
.platform-mac .logo {
  padding-top: 26px;
  height: calc(var(--header-height) + 26px);
  box-sizing: content-box;
}

.logo {
  overflow: hidden;
  white-space: nowrap;
  font-family: var(--font-family-display);
  height: var(--header-height);
  display: flex;
  align-items: center;
  padding: 0 20px;
  font-size: 20px;
  font-weight: 500;
  letter-spacing: -0.025em;
  color: var(--text-color);
}

.logo-icon {
  /* 头部圆形头像：44px 与 20px 字号的「小遥账单」视觉重量更匹配 */
  transition: width 0.22s ease, height 0.22s ease, margin-right 0.22s ease;
  width: 44px;
  height: 44px;
  margin-right: 12px;
  object-fit: contain;
}

.sidebar-toggle {
  margin-left: auto;
  width: 26px;
  height: 26px;
  flex: none;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--secondary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: default;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.sidebar-toggle:hover {
  background: rgba(0, 0, 0, 0.06);
  color: var(--text-color);
}

/* —— 收起态：64px 图标栏 —— */
.sidebar-collapsed .logo {
  flex-direction: column;
  justify-content: center;
  height: 96px;
  padding: 10px 0 6px;
  gap: 6px;
}

.sidebar-collapsed .logo-icon {
  width: 32px;
  height: 32px;
  margin-right: 0;
}

.sidebar-collapsed .logo span {
  display: none;
}

.sidebar-collapsed .sidebar-toggle {
  margin-left: 0;
}

.sidebar-collapsed .nav-item {
  justify-content: center;
  padding: 12px 0;
}

.sidebar-collapsed .nav-item span {
  display: none;
}

.sidebar-collapsed .nav-item .nav-icon {
  margin-right: 0;
}

.sidebar-collapsed .nav-item .nav-external {
  display: none;
}

.floating-menu.collapsed {
  padding: 0 8px;
}

.floating-menu.collapsed .filter-btn {
  justify-content: center;
  padding: 12px 0;
}

.floating-menu.collapsed .filter-btn span {
  display: none;
}

/* macOS 收起态仍要给左上角红绿灯留位置 */
.platform-mac.sidebar-collapsed .logo {
  padding-top: 30px;
  height: 108px;
}

.nav-menu {
  padding: 8px 0;
  flex: 1;
}

.nav-item {
  white-space: nowrap;
  overflow: hidden;
  padding: 12px 20px;
  margin: 4px 8px;
  display: flex;
  align-items: center;
  color: var(--text-color);
  text-decoration: none;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: var(--radius-sm);
  font-size: 14px;
  font-family: var(--font-family-text);
  letter-spacing: -0.016em;
}

.nav-item:hover {
  background: #E7E7EB;
  color: var(--primary-color);
}

.nav-item.active {
  background: #FFFFFF;
  color: var(--primary-color);
  font-weight: 500;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.nav-item-external {
  width: calc(100% - 16px);
  font: inherit;
  text-align: left;
  background: transparent;
  border: 0;
  cursor: pointer;
}

.nav-item-external .nav-external {
  flex: none;
  margin-left: auto;
  color: #8e8e93;
}

.nav-item .nav-icon {
  margin-right: 12px;
  flex: none;
}

/* 为每个菜单项图标添加颜色 */
.icon-home { color: #007AFF; }
.icon-yearly { color: #5856D6; }
.icon-share { color: #007AFF; }
.icon-monthly { color: #34C759; }
.icon-category { color: #FF9500; }
.icon-time { color: #AF52DE; }
.icon-insights { color: #FFCC00; }
.icon-transactions { color: #30B0C7; }
.icon-agent { color: #0A84FF; }
.icon-claw { color: #007AFF; }
.icon-settings { color: #8E8E93; }
.icon-author { color: #FF2D55; }

/* 页面标题区：各页面的 .page-header/.category-page-header 固定到窗口顶部，
   与侧边栏、窗口按钮组成统一标题栏；滚动时不再跟随内容。 */
.content :deep(.page-header),
.content :deep(.category-page-header) {
  position: fixed;
  top: 0;
  left: var(--sb-current-width);
  right: 0;
  height: 56px;
  min-height: 56px;
  margin: 0;
  /* 右侧给窗口按钮留 108px */
  padding: 0 108px 0 20px;
  z-index: 2400;
  background: var(--bg-color);
  border-bottom: 1px solid var(--border-color);
  align-items: center;
  box-sizing: border-box;
}

.content :deep(.page-header .page-title),
.content :deep(.category-page-header .page-title) {
  font-size: var(--font-size-xl);
  pointer-events: none;
}

/* 设置页 / AI Agent 页：标题与说明同一行，避免超出 56px */
.content :deep(.page-header-agent) {
  flex-direction: row;
  gap: 16px;
}

.content :deep(.page-header-agent .page-subtitle) {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  pointer-events: none;
}

.content {
  flex: 1;
  min-width: 0;
  /* 侧边栏改为绝对定位，内容区让出它的宽度 */
  margin-left: var(--sb-current-width);
  min-height: 0;
  transition: margin-left 0.22s ease;
  padding: 20px;
  background: var(--bg-color);
  overflow-y: auto;
}

/* 演示模式横幅 */
.demo-banner {
  margin-left: var(--sb-current-width);
  background: #FFF8E1;
  color: #F57F17;
  padding: 10px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #FFE0B2;
  position: relative;
  flex: none;
  z-index: 1001;
}

.demo-content {
  display: flex;
  align-items: center;
  gap: 8px;
}

.demo-content i {
  font-size: 16px;
}

.exit-demo-btn {
  background: #F57F17;
  color: white;
  border: none;
  padding: 6px 16px;
  border-radius: var(--radius-sm);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.exit-demo-btn:hover {
  background: #E65100;
}

/* 全局加载状态 */
/* 加载态改成内容区顶部的一条进度条：不遮挡页面、不盖住侧边栏与窗口按钮 */
.global-loader {
  position: fixed;
  top: 56px;
  left: var(--sb-current-width);
  right: 0;
  height: 3px;
  overflow: hidden;
  pointer-events: none;
  background: transparent;
  z-index: 1500;
}

.global-loader .loader-content {
  position: relative;
  width: 100%;
  height: 100%;
}

.global-loader .spinner {
  position: absolute;
  top: 0;
  left: 0;
  width: 35%;
  height: 100%;
  border: none;
  border-radius: 0;
  background: linear-gradient(90deg, rgba(0, 122, 255, 0), #007aff, rgba(0, 122, 255, 0));
  animation: loader-slide 1.1s ease-in-out infinite;
}

.global-loader .loader-text {
  display: none;
}

@keyframes loader-slide {
  from {
    transform: translateX(-100%);
  }
  to {
    transform: translateX(390%);
  }
}

.loader-content {
  text-align: center;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid rgba(255, 255, 255, 0.2);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin: 0 auto 16px;
}

.loader-text {
  color: white;
  font-size: 14px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

/* 浮动筛选菜单 */
.floating-menu {
  position: fixed;
  left: 0;
  bottom: 24px;
  width: var(--sb-current-width);
  padding: 0 12px;
  transition: width 0.22s ease;
  display: flex;
  flex-direction: column;
  gap: 8px;
  /* 高于侧边栏，否则会被全高侧栏盖住 */
  z-index: 2600;
}

.filter-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  border: none;
  border-radius: var(--radius-md);
  background: var(--card-bg);
  color: var(--text-color);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  width: 100%;
  box-shadow: var(--shadow-float);
}

.filter-btn:hover {
  transform: translateY(-1px);
  box-shadow: var(--shadow-hover);
}

.filter-btn.active {
  background: var(--primary-color);
  color: white;
}

.filter-btn svg {
  flex: none;
}

/* 响应式调整 */
@media (max-width: 768px) {
  .content {
    margin-left: 0;
    padding: 16px;
  }

  .sidebar {
    width: 60px;
    transform: translateX(-100%);
    transition: transform 0.3s ease;
  }

  .sidebar:hover,
  .sidebar.active {
    transform: translateX(0);
    width: 240px;
  }

  .nav-item span,
  .nav-external {
    display: none;
  }

  .sidebar:hover .nav-item span,
  .sidebar.active .nav-item span {
    display: inline;
  }

  .logo span {
    display: none;
  }

  .sidebar:hover .logo span,
  .sidebar.active .logo span {
    display: inline;
  }

  .demo-banner {
    flex-direction: column;
    gap: 8px;
    text-align: center;
  }

  /* 移动端隐藏筛选按钮文本 */
  .filter-btn span {
    display: none;
  }

  .sidebar:hover .filter-btn span,
  .sidebar.active .filter-btn span {
    display: inline;
  }

  .filter-btn {
    justify-content: center;
  }
}
</style>
