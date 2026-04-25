<template>
  <template v-if="authStore.isWidget">
    <router-view />
  </template>

  <div v-else class="app-container">
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
          <span v-if="mod.isPro && !authStore.isPro && !isCollapsed" class="pro-badge">PRO</span>
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
import { useAuthStore } from './stores/auth'; // 🌟 1. 引入真实的授权仓库

// 🌟 核心突破：摒弃所有可能存在延迟的 API，直接在 Setup 阶段读取浏览器底层地址栏！
// const isWidget = window.location.hash.includes('widget');

const workspaceStore = useWorkspaceStore();
const settingsStore = useSettingsStore();
const isCollapsed = ref(false); // 控制侧边栏折叠状态
const authStore = useAuthStore(); // 🌟 2. 实例化授权仓库

const mainModules = computed(() => appModules.filter(m => m.group === 'main'));
const settingsModules = computed(() => appModules.filter(m => m.group === 'settings'));
// let isWidget = window.location.hash.includes('widget');
onMounted(async () => {
  // 1. 🛡️ 铁穹第一步：建立安全会话 (最高优先级)
  // 必须拿到 sessionToken，否则后续所有调用都会被后端熔断拦截
  await authStore.initSecureSession();

  // 2. 🛡️ 铁穹第二步：环境初始化 (仅在主窗口模式下)
  if (!authStore.isWidget) {
    try {
      // 🌟 防御性检查：如果工作区尚未加载，执行加载
      if (!workspaceStore.currentWorkspace) {
        await workspaceStore.initWorkspace();
      }

      // 同步用户偏好设置 (如自动保存开关、并发数等)
      await settingsStore.fetchSettings();

      console.log("🚀 EasyCut 全局环境引导完成");
    } catch (e) {
      console.error("环境引导失败，系统将尝试降级运行:", e);
    }
  }

  // 3. 🛡️ 释放渲染大门
  authStore.isInitialized = true;
});
</script>

<style>
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