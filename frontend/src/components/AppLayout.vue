<template>
  <!-- 自定义标题栏：Windows 无边框后由它提供拖拽区与窗口按钮；macOS 只留红绿灯位 -->
  <div
    class="titlebar"
    :class="{ 'titlebar-mac': isMac }"
    data-tauri-drag-region
    @dblclick="toggleMaximize"
  >
    <div class="titlebar-controls" v-if="!isMac">
      <button class="titlebar-btn" aria-label="最小化" title="最小化" @click="minimizeWindow">
        <i class="fas fa-window-minimize"></i>
      </button>
      <button
        class="titlebar-btn"
        :aria-label="isMaximized ? '还原' : '最大化'"
        :title="isMaximized ? '还原' : '最大化'"
        @click="toggleMaximize"
      >
        <i class="fas" :class="isMaximized ? 'fa-window-restore' : 'fa-window-maximize'"></i>
      </button>
      <button class="titlebar-btn titlebar-close" aria-label="关闭" title="关闭" @click="closeWindow">
        <i class="fas fa-times"></i>
      </button>
    </div>
  </div>

  <!-- 演示模式横幅 -->
  <div v-if="sessionStore.isDemo" class="demo-banner">
    <div class="demo-content">
      <i class="fas fa-flask"></i>
      <span>您正在浏览示例数据（仅供演示功能预览）</span>
    </div>
    <button @click="exitDemoMode" class="exit-demo-btn">退出演示</button>
  </div>

  <div class="app-container">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="logo">
        <img src="/images/logo_128.png" alt="小遥账单助手" class="logo-icon" />
        <span>小遥账单</span>
      </div>
      <nav class="nav-menu">
        <router-link to="/" class="nav-item" :class="{ active: $route.path === '/' }">
          <i class="fas fa-home icon-home"></i>
          <span>首页</span>
        </router-link>
        <router-link to="/yearly" class="nav-item" :class="{ active: $route.path === '/yearly' }">
          <i class="fas fa-calendar-alt icon-yearly"></i>
          <span>年度总览</span>
        </router-link>
        <router-link to="/monthly" class="nav-item" :class="{ active: $route.path === '/monthly' }">
          <i class="fas fa-chart-line icon-monthly"></i>
          <span>月度分析</span>
        </router-link>
        <router-link to="/category" class="nav-item" :class="{ active: $route.path === '/category' }">
          <i class="fas fa-tags icon-category"></i>
          <span>分类分析</span>
        </router-link>
        <router-link to="/time" class="nav-item" :class="{ active: $route.path === '/time' }">
          <i class="fas fa-clock icon-time"></i>
          <span>时间分析</span>
        </router-link>
        <router-link to="/insights" class="nav-item" :class="{ active: $route.path === '/insights' }">
          <i class="fas fa-lightbulb icon-insights"></i>
          <span>消费洞察</span>
        </router-link>
        <router-link to="/transactions" class="nav-item" :class="{ active: $route.path === '/transactions' }">
          <i class="fas fa-receipt icon-transactions"></i>
          <span>交易记录</span>
        </router-link>
        <router-link to="/agent" class="nav-item" :class="{ active: $route.path === '/agent' }">
          <i class="fas fa-robot icon-agent"></i>
          <span>AI Agent 接入</span>
        </router-link>
        <router-link to="/settings" class="nav-item" :class="{ active: $route.path === '/settings' }">
          <i class="fas fa-cog icon-settings"></i>
          <span>设置</span>
        </router-link>
        <router-link to="/about-author" class="nav-item" :class="{ active: $route.path === '/about-author' }">
          <i class="fas fa-user-circle icon-author"></i>
          <span>关于作者</span>
        </router-link>
      </nav>
    </aside>

    <!-- 主内容区域 -->
    <main class="content">
      <router-view />
    </main>
  </div>

  <!-- 筛选菜单 (独立于侧边栏) -->
  <div class="floating-menu">
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'all' }"
      @click="setFilter('all')"
    >
      <i class="fas fa-list-ul"></i>
      <span>全部交易</span>
    </button>
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'large' }"
      @click="setFilter('large')"
    >
      <i class="fas fa-coins"></i>
      <span>仅大额交易</span>
    </button>
    <button
      class="filter-btn"
      :class="{ active: filterStore.currentFilter === 'small' }"
      @click="setFilter('small')"
    >
      <i class="fas fa-coffee"></i>
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
/* 自定义标题栏：固定在最顶部，整条可拖拽 */
.titlebar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  height: 34px;
  z-index: 3000;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  background: #F2F2F4;
  border-bottom: 1px solid var(--border-color);
  user-select: none;
}

/* macOS：原生红绿灯浮在左上角，避开即可 */
.titlebar-mac {
  justify-content: flex-start;
  padding-left: 82px;
}

.titlebar-controls {
  display: flex;
  align-items: stretch;
  height: 100%;
}

.titlebar-btn {
  width: 46px;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text-color);
  font-size: 11px;
  cursor: default;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: background-color 0.15s ease;
}

.titlebar-btn:hover {
  background: var(--hover-bg);
}

.titlebar-btn:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: -2px;
}

.titlebar-close:hover {
  background: #E81123;
  color: #fff;
}

.app-container {
  display: flex;
  min-height: 100vh;
}

.sidebar {
  position: fixed;
  left: 0;
  top: 34px;
  bottom: 0;
  width: var(--sidebar-width);
  /* 比卡片白底略灰，避免整块侧栏发白 */
  background: #F2F2F4;
  border-right: 1px solid var(--border-color);
  z-index: 1000;
  overflow-y: auto;
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

.nav-item i {
  margin-right: 12px;
  width: 20px;
  text-align: center;
  font-size: 16px;
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

.content {
  flex: 1;
  margin-left: var(--sidebar-width);
  /* 顶部让出固定的自定义标题栏 */
  padding: 54px 20px 20px;
  background: var(--bg-color);
  min-height: calc(100vh - 34px);
}

/* 演示横幅已经在标题栏下方，主内容不再重复让位 */
.demo-banner + .app-container .content {
  padding-top: 20px;
}

/* 演示模式横幅 */
.demo-banner {
  background: #FFF8E1;
  color: #F57F17;
  padding: 10px 20px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #FFE0B2;
  position: sticky;
  top: 34px;
  z-index: 1001;
  margin-top: 34px;
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

.filter-btn i {
  font-size: 16px;
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
