<!-- src/components/ExportResultModal.vue -->
<template>
  <transition name="zoom-in">
    <div class="export-result-overlay" v-if="modelValue" @click.self="close">
      <div class="export-modal-content">
        <div class="modal-header">
          <div class="header-left">
            <h2>🎉 导出完成</h2>
            <!-- 🌟 新增：优雅的耗时徽章 -->
            <transition name="pop-in">
              <span v-if="costTime" class="time-badge">
                ⏱️ 极速耗时: <strong>{{ costTime }}</strong> 秒
              </span>
            </transition>
          </div>
          <button class="icon-btn" @click="close">✖</button>
        </div>
        <div class="modal-body">
          <textarea class="log-textarea" readonly :value="logs"></textarea>
        </div>
        <div class="modal-footer">
          <button class="secondary-btn" @click="close">关闭</button>
          <button class="primary-btn" @click="openFolder">📂 打开所在文件夹</button>
        </div>
      </div>
    </div>
  </transition>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  modelValue: boolean;
  logs: string;
  exportDir: string;
  costTime?: string; // 🌟 新增：接收父组件传来的耗时
}>();

const emit = defineEmits(['update:modelValue']);

function close() {
  emit('update:modelValue', false);
}

async function openFolder() {
  if (props.exportDir) {
    try {
      await invoke('open_folder', { path: props.exportDir });
      close();
    } catch (e) {
      alert("无法打开目录");
    }
  }
}
</script>

<style scoped>
.export-result-overlay { position: fixed; inset: 0; background: rgba(15, 23, 42, 0.75); backdrop-filter: blur(4px); z-index: 9999; display: flex; justify-content: center; align-items: center; }
.export-modal-content { background: #ffffff; width: 600px; max-width: 90%; border-radius: 12px; box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5); overflow: hidden; display: flex; flex-direction: column; }

/* 🌟 头部布局优化 */
.modal-header { padding: 1.25rem 1.5rem; border-bottom: 1px solid #e5e7eb; display: flex; justify-content: space-between; align-items: center; }
.header-left { display: flex; align-items: center; gap: 1rem; }
.header-left h2 { margin: 0; font-size: 1.25rem; color: #111827; }

/* 🌟 极速耗时徽章样式 */
.time-badge {
  background: #ecfdf5; color: #059669; padding: 4px 12px;
  border-radius: 20px; font-size: 0.85rem; font-weight: 500;
  border: 1px solid #a7f3d0; display: inline-flex; align-items: center; gap: 4px;
}
.time-badge strong { font-size: 1rem; font-family: monospace; font-weight: 800; }

.modal-body { padding: 1.5rem; background: #f9fafb; }
.log-textarea { width: 100%; height: 250px; padding: 1rem; border-radius: 8px; border: 1px solid #d1d5db; background: #1f2937; color: #10b981; font-family: monospace; font-size: 0.9rem; outline: none; resize: none; }
.modal-footer { padding: 1rem 1.5rem; border-top: 1px solid #e5e7eb; display: flex; justify-content: flex-end; gap: 1rem; }

.zoom-in-enter-active, .zoom-in-leave-active { transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1); }
.zoom-in-enter-from, .zoom-in-leave-to { opacity: 0; transform: translate(-50%, -50%) scale(0.9); }
.pop-in-enter-active { animation: popIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
@keyframes popIn {
  0% { transform: scale(0.8); opacity: 0; }
  100% { transform: scale(1); opacity: 1; }
}
</style>