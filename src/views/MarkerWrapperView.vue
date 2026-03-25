<template>
  <div class="wrapper-container">
    <div class="tabs-header">
      <div class="tabs-nav">
        <button class="tab-btn active">
          <span class="icon">⌨️</span> 手工精确打轴
        </button>
        <button class="tab-btn disabled" title="Pro 版本功能，敬请期待">
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
</script>

<style scoped>
.wrapper-container { height: 100%; display: flex; flex-direction: column; padding: 2rem; box-sizing: border-box;}
.tabs-header { margin-bottom: 1.5rem; flex-shrink: 0; }
.tabs-nav { display: inline-flex; background: #e5e7eb; padding: 4px; border-radius: 10px; gap: 4px;}
.tab-btn { background: transparent; border: none; padding: 0.6rem 1.2rem; border-radius: 8px; font-weight: 600; color: #4b5563; cursor: pointer; transition: all 0.2s; font-size: 0.95rem;}
.tab-btn.active { background: white; color: #2563eb; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
.tab-btn.disabled { opacity: 0.6; cursor: not-allowed; }
.pro-tag { font-size: 0.6rem; background: #f59e0b; color: white; padding: 1px 4px; border-radius: 4px; margin-left: 4px; vertical-align: top;}
.module-content { flex: 1; min-height: 0; position: relative;}
</style>