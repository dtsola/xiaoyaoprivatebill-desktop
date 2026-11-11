<template>
  <div class="app-shell" :class="{ 'platform-mac': isMac }">
    <!-- 顶部拖拽条：只覆盖内容区，侧边栏从窗口最顶开始（对齐 xiaoyaotime） -->
    <div class="drag-strip" data-tauri-drag-region @dblclick="toggleMaximize"></div>

    <!-- 窗口按钮：26px 圆角按钮，右上角 -->
    <div class="window-controls" v-if="!isMac">
      <button class="window-btn" aria-label="最小化" title="最小化" @click="minimizeWindow">
        <Minus :size="14" />
      </button>
      <button
        class="window-btn"
        :aria-label="isMaximized ? '还原' : '最大化'"
        :title="isMaximized ? '还原' : '最大化'"
        @click="toggleMaximize"
      >
        <Copy v-if="isMaximized" :size="12" />
        <Square v-else :size="11" />
      </button>
      <button class="window-btn window-close" aria-label="关闭" title="关闭" @click="closeWindow">
        <X :size="14" />
      </button>
    </div>

    <!-- 演示模式横幅 -->
    <div v-if="sessionStore.isDemo" class="demo-banner">
      <div class="demo-content">
        <FlaskConical :size="16" />
        <span>您正在浏览示例数据（仅供演示功能预览）</span>
      </div>
      <button @click="exitDemoMode" class="exit-demo-btn">退出演示</button>
    </div>

    <div class="app-container">
      <!-- 侧边栏：整列贴到窗口顶部，LOGO 区域可拖拽窗口 -->
      <aside class="sidebar">
        <div class="logo" data-tauri-drag-region>
          <img
            src="/images/logo_128.png"
            alt="小遥账单助手"
            class="logo-icon"
            data-tauri-drag-region
          />
          <span data-tauri-drag-region>小遥账单</span>
        </div>
        <nav class="nav-menu">
          <router-link to="/" class="nav-item" :class="{ active: $route.path === '/' }">
            <House class="nav-icon icon-home" :size="18" />
            <span>首页</span>
          </router-link>
          <router-link to="/yearly" class="nav-item" :class="{ active: $route.path === '/yearly' }">
            <CalendarDays class="nav-icon icon-yearly" :size="18" />
            <span>年度总览</span>
          </router-link>
          <router-link to="/monthly" class="nav-item" :class="{ active: $route.path === '/monthly' }">
            <ChartLine class="nav-icon icon-monthly" :size="18" />
            <span>月度分析</span>
          </router-link>
          <router-link to="/category" class="nav-item" :class="{ active: $route.path === '/category' }">
            <Tags class="nav-icon icon-category" :size="18" />
            <span>分类分析</span>
          </router-link>
          <router-link to="/time" class="nav-item" :class="{ active: $route.path === '/time' }">
            <Clock class="nav-icon icon-time" :size="18" />
            <span>时间分析</span>
          </router-link>
          <router-link to="/insights" class="nav-item" :class="{ active: $route.path === '/insights' }">
            <Lightbulb class="nav-icon icon-insights" :size="18" />
            <span>消费洞察</span>
          </router-link>
          <router-link to="/transactions" class="nav-item" :class="{ active: $route.path === '/transactions' }">
            <Receipt class="nav-icon icon-transactions" :size="18" />
            <span>交易记录</span>
          </router-link>
          <router-link to="/agent" class="nav-item" :class="{ active: $route.path === '/agent' }">
            <Bot class="nav-icon icon-agent" :size="18" />
            <span>AI Agent 接入</span>
          </router-link>
          <router-link to="/settings" class="nav-item" :class="{ active: $route.path === '/settings' }">
            <Settings class="nav-icon icon-settings" :size="18" />
            <span>设置</span>
          </router-link>
          <router-link to="/about-author" class="nav-item" :class="{ active: $route.path === '/about-author' }">
            <CircleUserRound class="nav-icon icon-author" :size="18" />
            <span>关于作者</span>
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
  <div class="floating-menu">
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'all' }"
      @click="setFilter('all')"
    >
      <List :size="16" />
      <span>全部交易</span>
    </button>
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'large' }"
      @click="setFilter('large')"
    >
      <Coins :size="16" />
      <span>仅大额交易</span>
    </button>
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'small' }"
      @click="setFilter('small')"
    >
      <Coffee :size="16" />
      <span>仅小额交易</span>
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
      <div class="loader-text">加载中...</div>
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import {
  Bot,
  CalendarDays,
  ChartLine,
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
  Receipt,
  Settings,
  Square,
  Tags,
  X
} from 'lucide-vue-next'
import { useUiStore } from '@/stores/ui'
import { useSessionStore } from '@/stores/session'
import { useFilterStore } from '@/stores/filter'
import Toast from '@/components/common/Toast.vue'
import Modal from '@/components/common/Modal.vue'

const router = useRouter()
const uiStore = useUiStore()
const sessionStore = useSessionStore()
const filterStore = useFilterStore()

let unlistenCloseHint = null

// 窗口外观：Windows 由 Rust 去掉原生边框，这里渲染拖拽区与按钮；
// macOS 保留原生红绿灯，只需留出左侧空间。
const isMac = typeof navigator !== 'undefined' && /Mac/i.test(navigator.userAgent)
const isMaximized = ref(false)
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
  unlistenCloseHint = await listen('desktop://close-to-tray-hint', () => {
    uiStore.showSuccess('已最小化到托盘，可从托盘菜单退出')
  })

  // 最大化状态跟随系统（双击拖拽区、Win+↑、任务栏菜单都会触发 resize）
  const win = windowHandle()
  if (win && !isMac) {
    await syncMaximized()
    unlistenResize = await win.onResized(syncMaximized)
  }
})

onUnmounted(() => {
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
  if (confirm('确定要退出演示模式吗？')) {
    try {
      await sessionStore.exitDemoMode()
      uiStore.showSuccess('已退出演示模式')
      router.replace('/')
    } catch (error) {
      uiStore.showError('退出失败: ' + error.message)
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
</script>

<style scoped>
/* 完全复制老前端的样式 */
/* 应用外壳：固定视口高度；顶部 36px 留给拖拽条与窗口按钮 */
.app-shell {
  position: relative;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding-top: 36px;
}

/* 拖拽条只覆盖内容区：侧边栏占满窗口高度，LOGO 区也可拖拽 */
.drag-strip {
  position: absolute;
  top: 0;
  left: var(--sidebar-width);
  right: 96px;
  height: 36px;
  z-index: 2000;
  user-select: none;
}

/* macOS：原生红绿灯在左上角，拖拽条从最左侧开始也无冲突 */
.platform-mac .drag-strip {
  left: 0;
}

/* 窗口按钮：与 xiaoyaotime 对齐（26px、右上角、圆角悬停） */
.window-controls {
  position: absolute;
  top: 0;
  right: 0;
  height: 36px;
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
  width: var(--sidebar-width);
  height: 100%;
  display: flex;
  flex-direction: column;
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
  width: 44px;
  height: 44px;
  margin-right: 12px;
  object-fit: contain;
}

.nav-menu {
  padding: 8px 0;
  flex: 1;
}

.nav-item {
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

.nav-item .nav-icon {
  margin-right: 12px;
  flex: none;
}

/* 为每个菜单项图标添加颜色 */
.icon-home { color: #007AFF; }
.icon-yearly { color: #5856D6; }
.icon-monthly { color: #34C759; }
.icon-category { color: #FF9500; }
.icon-time { color: #AF52DE; }
.icon-insights { color: #FFCC00; }
.icon-transactions { color: #30B0C7; }
.icon-agent { color: #0A84FF; }
.icon-settings { color: #8E8E93; }
.icon-author { color: #FF2D55; }

/* 页面头部是全局 sticky(top:0)，滚动容器换了以后需要抵消内容区 20px 内边距，
   否则滚动时标题栏与页面头部之间会露出 20px 间隙。 */
.content :deep(.page-header),
.content :deep(.category-page-header) {
  top: -20px;
}

.content {
  flex: 1;
  min-width: 0;
  /* 侧边栏改为绝对定位，内容区让出它的宽度 */
  margin-left: var(--sidebar-width);
  min-height: 0;
  padding: 20px;
  background: var(--bg-color);
  overflow-y: auto;
}

/* 演示模式横幅 */
.demo-banner {
  margin-left: var(--sidebar-width);
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
.global-loader {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
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
  width: var(--sidebar-width);
  padding: 0 12px;
  display: flex;
  flex-direction: column;
  gap: 8px;
  z-index: 1200;
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

  .nav-item span {
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
