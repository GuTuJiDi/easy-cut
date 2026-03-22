<template>
  <div class="container">
    <h1>易剪 (EasyCut) V1.0 - 开发环境测试</h1>
    <button @click="testEngine">检测底层切割引擎</button>
    <p v-if="engineStatus" class="status">{{ engineStatus }}</p>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

const engineStatus = ref('');

async function testEngine() {
  try {
    // 调用 Rust 后端的 check_ffmpeg_status 方法
    const result = await invoke<string>('check_ffmpeg_status');
    engineStatus.value = `✅ ${result}`;
  } catch (error) {
    engineStatus.value = `❌ 错误: ${error}`;
  }
}
</script>

<style scoped>
.container {
  padding: 2rem;
  font-family: sans-serif;
}
.status {
  margin-top: 1rem;
  padding: 1rem;
  background: #f0f0f0;
  border-radius: 4px;
}
</style>