<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="brand">
        <span class="logo">✂️</span>
        <h1 class="app-name">易剪 EasyCut</h1>
      </div>

      <nav class="nav-menu">
        <router-link to="/split" class="nav-item">
          <span class="icon">🔪</span> 基础无损分割
        </router-link>
        <router-link to="/marker" class="nav-item">
          <span class="icon">🏷️</span> 智能打轴标记
        </router-link>
        <router-link to="/audio" class="nav-item">
          <span class="icon">🎵</span> 音频极速分离
          <span v-if="!workspaceStore.isProVersion" class="pro-badge">PRO</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <router-link to="/settings" class="nav-item settings-link">
          <span class="icon">⚙️</span> 全局设置
        </router-link>
      </div>
    </aside>

    <main class="main-content">
      <router-view v-slot="{ Component }">
        <transition name="fade" mode="out-in">
          <component :is="Component" />
        </transition>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useWorkspaceStore } from './stores/workspace';

const workspaceStore = useWorkspaceStore();

onMounted(() => {
  // 软件启动时，从 Rust 后端拉取当前工作区路径并存储到全局
  workspaceStore.initWorkspace();
});
</script>

<style>
/* 全局重置与基础样式 */
html, body { margin: 0; padding: 0; height: 100%; font-family: 'Segoe UI', system-ui, sans-serif; background-color: #f3f4f6; }
#app { height: 100vh; }

.app-container {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* 侧边栏样式 */
.sidebar {
  width: 240px;
  background-color: #ffffff;
  border-right: 1px solid #e5e7eb;
  display: flex;
  flex-direction: column;
}

.brand {
  padding: 1.5rem;
  display: flex;
  align-items: center;
  gap: 10px;
  border-bottom: 1px solid #f3f4f6;
}

.brand .logo { font-size: 1.5rem; }
.app-name { margin: 0; font-size: 1.25rem; color: #111827; }

.nav-menu {
  flex: 1;
  padding: 1rem 0;
  display: flex;
  flex-direction: column;
  gap: 0.25rem;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 0.75rem 1.5rem;
  color: #4b5563;
  text-decoration: none;
  font-weight: 500;
  transition: all 0.2s;
}

.nav-item:hover { background-color: #f9fafb; color: #2563eb; }

/* 路由被激活时的样式 */
.nav-item.router-link-active {
  background-color: #eff6ff;
  color: #2563eb;
  border-right: 3px solid #2563eb;
}

.pro-badge {
  margin-left: auto;
  font-size: 0.65rem;
  background-color: #f59e0b;
  color: white;
  padding: 2px 6px;
  border-radius: 10px;
  font-weight: bold;
}

.sidebar-footer { padding: 1rem 0; border-top: 1px solid #e5e7eb; }

/* 主内容区 */
.main-content {
  flex: 1;
  overflow-y: auto;
  padding: 2rem;
}

/* 页面切换动画 */
.fade-enter-active, .fade-leave-active { transition: opacity 0.15s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>