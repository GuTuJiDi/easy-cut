<template>
  <div class="app-container">
    <aside class="sidebar" :class="{ 'is-collapsed': isCollapsed }">
      <div class="brand">
        <div class="logo-box">✂️</div>
        <transition name="fade-fast">
          <div class="brand-text" v-show="!isCollapsed">
            <h1 class="app-name">易剪 EasyCut</h1>
            <span class="version">V1.0.0</span>
          </div>
        </transition>
        <button class="collapse-btn" @click="isCollapsed = !isCollapsed">
          {{ isCollapsed ? '❯' : '❮' }}
        </button>
      </div>

      <nav class="nav-menu">
        <router-link v-for="mod in mainModules" :key="mod.id" :to="mod.disabled ? '' : mod.path" class="nav-item" :class="{ 'is-disabled': mod.disabled }" :title="isCollapsed ? mod.name : ''">
          <span class="icon">{{ mod.icon }}</span>
          <span class="mod-name" v-show="!isCollapsed">{{ mod.name }}</span>
          <span v-if="mod.isPro && !workspaceStore.isProVersion && !isCollapsed" class="pro-badge">PRO</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <router-link v-for="mod in settingsModules" :key="mod.id" :to="mod.path" class="nav-item settings-link" :title="isCollapsed ? mod.name : ''">
          <span class="icon">{{ mod.icon }}</span>
          <span class="mod-name" v-show="!isCollapsed">{{ mod.name }}</span>
        </router-link>
      </div>
    </aside>

    <main class="main-content">
      <router-view v-slot="{ Component }">
        <transition name="fade-page" mode="out-in">
          <keep-alive><component :is="Component" /></keep-alive>
        </transition>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useWorkspaceStore } from './stores/workspace';
import { useSettingsStore } from './stores/settings';
import { appModules } from './config/modules';

const workspaceStore = useWorkspaceStore();
const settingsStore = useSettingsStore();
const isCollapsed = ref(false); // 控制侧边栏折叠状态

const mainModules = computed(() => appModules.filter(m => m.group === 'main'));
const settingsModules = computed(() => appModules.filter(m => m.group === 'settings'));

onMounted(() => {
  workspaceStore.initWorkspace();
  settingsStore.fetchSettings(); // 启动时全局拉取设置
});
</script>

<style>
/* ========================================= */
/* 🌍 核心通用全局样式 (V2.0 布局规范)           */
/* 用于解决通用 Class 在具体页面组件内不起效的问题 */
/* ========================================= */

/* 1. 基础重置 */
html, body { margin: 0; padding: 0; height: 100%; font-family: 'Segoe UI', system-ui, sans-serif; background-color: #f3f4f6; overflow: hidden; }
#app { height: 100vh; }
.app-container { display: flex; height: 100vh; overflow: hidden; width: 100vw; }

/* 2. 主内容区核心自适应约束 (修复挤压问题) */
.main-content {
  flex: 1;
  overflow-y: hidden; /* 让子组件自己负责垂直滚动，防止全局和局部滚动条冲突 */
  position: relative;
  background: #f3f4f6;
  min-width: 0; /* 极其重要：允许父容器压缩，让子组件应用 min-width 底线限制 */
}

/* 3. 🛡️ 易剪 V2.0 通用布局 Class 规范 */
/* 以后任何新页面，请直接套用这些 Class，绝不需要在组件内再调布局 */

.view-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  padding: 2rem; /* 统一间距规范 */
  box-sizing: border-box;
  animation: fadeIn 0.3s ease;
  overflow-y: auto; /* 让最外层的容器负责统一滚动 */
}

/* 通用 Header 规范 (移除 margin-left) */
.view-header { margin-bottom: 2rem; flex-shrink: 0; }
.view-title { font-size: 1.75rem; font-weight: 700; color: #111827; margin: 0 0 0.25rem 0; letter-spacing: -0.5px;}
.view-subtitle { color: #6b7280; margin: 0; font-size: 0.95rem; }

/* 通用 卡片（Card）样式规范 */
.card {
  background: #ffffff;
  border-radius: 12px;
  border: 1px solid #e5e7eb;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  box-shadow: 0 1px 3px rgba(0,0,0,0.02);
}
.card.padded { padding: 1.5rem; }

/* 通用 OSD / Toast / 过渡效果 */
@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
.fade-page-enter-active, .fade-page-leave-active { transition: opacity 0.2s ease, transform 0.2s ease; }
.fade-page-enter-from { opacity: 0; transform: translateY(10px); }
.fade-page-leave-to { opacity: 0; transform: translateY(-10px); }
.fade-fast-enter-active, .fade-fast-leave-active { transition: opacity 0.15s ease-in-out; }
.fade-fast-enter-from, .fade-fast-leave-to { opacity: 0; }

/* ========================================= */
/* 🧱 App Shell 独有样式                      */
/* ========================================= */

.sidebar { width: 240px; background-color: #fcfcfd; border-right: 1px solid #e5e7eb; display: flex; flex-direction: column; z-index: 10; transition: width 0.3s cubic-bezier(0.4, 0, 0.2, 1); white-space: nowrap; overflow: hidden; position: relative;}
.sidebar.is-collapsed { width: 72px; }

.brand { padding: 1rem 1rem; display: flex; align-items: center; gap: 12px; border-bottom: 1px solid transparent; height: 70px; flex-shrink: 0; box-sizing: border-box; position: relative;}
.logo-box { font-size: 1.5rem; background: #eff6ff; min-width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; border-radius: 10px; border: 1px solid #bfdbfe; flex-shrink: 0; }
.brand-text { display: flex; flex-direction: column; }
.app-name { margin: 0; font-size: 1.2rem; color: #111827; font-weight: 800;}
.version { font-size: 0.7rem; color: #9ca3af; font-weight: bold; }

.collapse-btn { position: absolute; right: -12px; top: 25px; width: 24px; height: 24px; background: white; border: 1px solid #e5e7eb; border-radius: 50%; cursor: pointer; color: #6b7280; display: flex; align-items: center; justify-content: center; font-size: 10px; box-shadow: 0 1px 2px rgba(0,0,0,0.05); z-index: 20; }
.collapse-btn:hover { background: #f3f4f6; color: #111827;}

.nav-menu { flex: 1; padding: 0 0.75rem; display: flex; flex-direction: column; gap: 0.35rem; margin-top: 1rem;}
.nav-item { display: flex; align-items: center; gap: 12px; padding: 0.75rem; color: #4b5563; text-decoration: none; font-weight: 600; font-size: 0.95rem; border-radius: 8px; transition: all 0.2s ease;}
.nav-item:hover:not(.is-disabled) { background-color: #f3f4f6; color: #111827; }
.nav-item.router-link-active { background-color: #ebf5ff; color: #2563eb; box-shadow: inset 3px 0 0 #2563eb; }
.icon { font-size: 1.2rem; min-width: 24px; text-align: center;}
.mod-name { flex: 1; }

.is-disabled { opacity: 0.5; cursor: not-allowed; }
.pro-badge { font-size: 0.65rem; background: linear-gradient(135deg, #f59e0b, #d97706); color: white; padding: 2px 6px; border-radius: 6px; font-weight: 900;}

.sidebar-footer { padding: 1rem 0.75rem; border-top: 1px solid #e5e7eb; }
</style>