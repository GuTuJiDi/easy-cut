<template>
  <div class="app-container">
    <aside class="sidebar">
      <div class="brand">
        <div class="logo-box">✂️</div>
        <div class="brand-text">
          <h1 class="app-name">易剪 EasyCut</h1>
          <span class="version">V1.0.0</span>
        </div>
      </div>

      <nav class="nav-menu">
        <router-link
            v-for="mod in mainModules"
            :key="mod.id"
            :to="mod.disabled ? '' : mod.path"
            class="nav-item"
            :class="{ 'is-disabled': mod.disabled }"
        >
          <span class="icon">{{ mod.icon }}</span>
          <span class="mod-name">{{ mod.name }}</span>
          <span v-if="mod.isPro && !workspaceStore.isProVersion" class="pro-badge">PRO</span>
        </router-link>
      </nav>

      <div class="sidebar-footer">
        <router-link
            v-for="mod in settingsModules"
            :key="mod.id"
            :to="mod.path"
            class="nav-item settings-link"
        >
          <span class="icon">{{ mod.icon }}</span>
          <span class="mod-name">{{ mod.name }}</span>
        </router-link>
      </div>
    </aside>

    <main class="main-content">
      <router-view v-slot="{ Component }">
        <transition name="fade-page" mode="out-in">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </transition>
      </router-view>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useWorkspaceStore } from './stores/workspace';
import { appModules } from './config/modules';

const workspaceStore = useWorkspaceStore();

// 计算属性：自动过滤出不同分组的模块
const mainModules = computed(() => appModules.filter(m => m.group === 'main'));
const settingsModules = computed(() => appModules.filter(m => m.group === 'settings'));

onMounted(() => {
  workspaceStore.initWorkspace();
});
</script>

<style>
/* 核心布局复用，做像素级优化 */
html, body { margin: 0; padding: 0; height: 100%; font-family: 'Segoe UI', system-ui, sans-serif; background-color: #f3f4f6; }
#app { height: 100vh; }
.app-container { display: flex; height: 100vh; overflow: hidden; }

/* 侧边栏极致美化 */
.sidebar { width: 240px; background-color: #fcfcfd; border-right: 1px solid #e5e7eb; display: flex; flex-direction: column; z-index: 10; box-shadow: 1px 0 5px rgba(0,0,0,0.02);}
.brand { padding: 1.5rem; display: flex; align-items: center; gap: 12px; border-bottom: 1px solid transparent; margin-bottom: 0.5rem;}
.logo-box { font-size: 1.6rem; background: #eff6ff; width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; border-radius: 10px; border: 1px solid #bfdbfe;}
.brand-text { display: flex; flex-direction: column; }
.app-name { margin: 0; font-size: 1.2rem; color: #111827; font-weight: 800;}
.version { font-size: 0.7rem; color: #9ca3af; font-weight: bold; font-family: monospace;}

.nav-menu { flex: 1; padding: 0 0.75rem; display: flex; flex-direction: column; gap: 0.35rem; }
.nav-item { display: flex; align-items: center; gap: 12px; padding: 0.75rem 1rem; color: #4b5563; text-decoration: none; font-weight: 600; font-size: 0.95rem; border-radius: 8px; transition: all 0.2s ease;}
.nav-item:hover:not(.is-disabled) { background-color: #f3f4f6; color: #111827; }
.nav-item.router-link-active { background-color: #ebf5ff; color: #2563eb; box-shadow: inset 3px 0 0 #2563eb; }
.icon { font-size: 1.1rem; }
.mod-name { flex: 1; }

.is-disabled { opacity: 0.5; cursor: not-allowed; }
.pro-badge { font-size: 0.65rem; background: linear-gradient(135deg, #f59e0b, #d97706); color: white; padding: 2px 6px; border-radius: 6px; font-weight: 900; letter-spacing: 0.5px; box-shadow: 0 2px 4px rgba(245, 158, 11, 0.3);}

.sidebar-footer { padding: 1rem 0.75rem; border-top: 1px solid #e5e7eb; }

/* 主内容区 */
.main-content { flex: 1; overflow-y: hidden; position: relative; background: #f3f4f6;}

/* 页面切换高级动画 */
.fade-page-enter-active, .fade-page-leave-active { transition: opacity 0.2s ease, transform 0.2s ease; }
.fade-page-enter-from { opacity: 0; transform: translateY(10px); }
.fade-page-leave-to { opacity: 0; transform: translateY(-10px); }
</style>