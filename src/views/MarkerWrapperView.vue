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
.module-content { flex: 1; min-height: 0; position: relative;}
</style>