<template>
  <div class="wrapper-container">
    <div class="tabs-header">
      <div class="tabs-nav">
        <button class="tab-btn active">
          <span class="icon">⌨️</span> 手工精确打轴
        </button>
        <!-- 🌟 PRO 权限动态绑定 -->
        <button
            class="tab-btn"
            :class="{ disabled: !authStore.isPro }"
            @click="handleAITabClick"
            :title="authStore.isPro ? '进入 AI 智能打轴 (即将开启)' : 'PRO 旗舰版专属功能'"
        >
          <span class="icon">🤖</span> AI 智能识别打轴 <span class="pro-tag">PRO</span>
        </button>
      </div>
    </div>

    <div class="module-content">
      <ManualMarker />
    </div>
  </div>
</template>

<script setup lang="ts">
import ManualMarker from '../components/ManualMarker.vue';
import { useAuthStore } from '../stores/auth'; // 🌟 引入鉴权
const authStore = useAuthStore();
// 🌟 拦截逻辑放在 Script 里，清爽且符合规范
function handleAITabClick() {
  if (!authStore.isPro) {
    alert('AI 智能识别为 PRO 旗舰版专属能力，请先前往【全局偏好设置】激活！');
  } else {
    // 未来如果实现了 AI 功能，这里写切换 Tab 的逻辑
    alert('AI 引擎即将就绪，敬请期待...');
  }
}
</script>

<style scoped>
.wrapper-container { height: 100%; display: flex; flex-direction: column; padding: 2rem; box-sizing: border-box;}
.module-content { flex: 1; min-height: 0; position: relative;}
</style>