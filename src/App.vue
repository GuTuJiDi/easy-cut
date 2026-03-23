<template>
  <div class="app-container">
    <header class="header">
      <div class="logo">✂️</div>
      <div class="title-area">
        <h1>易剪 (EasyCut)</h1>
        <p class="subtitle">极速无损视频处理引擎 v1.0</p>
      </div>
    </header>

    <main class="main-content">
      <div class="card">
        <div class="card-header">
          <h2>⏱️ 固定时长批量分割</h2>
          <p>将长视频按设定的时间（如每 5 分钟）切分成多个无损片段。</p>
        </div>

        <div class="card-body">
          <div class="form-group">
            <label>🎬 输入视频绝对路径</label>
            <input
                v-model="batchParams.inputPath"
                type="text"
                placeholder="例如: D:\Videos\source.mp4"
            />
          </div>

          <div class="form-group">
            <label>📁 输出基础路径与命名前缀</label>
            <input
                v-model="batchParams.baseOutputName"
                type="text"
                placeholder="例如: D:\Videos\Output\clip"
            />
            <span class="hint">系统将自动在末尾追加 _part1.mp4, _part2.mp4...</span>
          </div>

          <div class="form-group">
            <label>⏳ 每个片段时长 (秒)</label>
            <input
                v-model.number="batchParams.segmentDuration"
                type="number"
                min="1"
                placeholder="例如: 300"
            />
            <span class="hint">当前设置: {{ (batchParams.segmentDuration / 60).toFixed(1) }} 分钟 / 段</span>
          </div>

          <button class="primary-btn" @click="runBatchSplit" :disabled="isProcessing">
            <span v-if="isProcessing" class="spinner">⚙️</span>
            {{ isProcessing ? '引擎全速处理中...' : '🚀 开始批量分割' }}
          </button>

          <div v-if="resultLog" :class="['result-box', isError ? 'error' : 'success']">
            <pre>{{ resultLog }}</pre>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';

// 状态管理
const isProcessing = ref(false);
const resultLog = ref('');
const isError = ref(false);

// 表单数据绑定
const batchParams = reactive({
  inputPath: '',
  baseOutputName: '',
  segmentDuration: 300 // 默认 300 秒 (5分钟)
});

// 触发后端切割任务
async function runBatchSplit() {
  // 基础校验
  if (!batchParams.inputPath || !batchParams.baseOutputName || !batchParams.segmentDuration) {
    resultLog.value = "⚠️ 请填写完整的路径和时长参数！";
    isError.value = true;
    return;
  }

  isProcessing.value = true;
  resultLog.value = '';
  isError.value = false;

  try {
    // 调用 Bob 刚写好的闭环接口
    const result = await invoke<string>('batch_split_by_duration', {
      inputPath: batchParams.inputPath,
      baseOutputName: batchParams.baseOutputName,
      segmentDuration: batchParams.segmentDuration
    });

    // 成功后展示日志
    resultLog.value = `🎉 批量分割任务完成！\n\n${result}`;
  } catch (error) {
    // 捕获后端的 Err 返回
    resultLog.value = `❌ 发生错误:\n${error}`;
    isError.value = true;
  } finally {
    isProcessing.value = false;
  }
}
</script>

<style scoped>
/* ===== 现代化 UI 样式定义 ===== */
.app-container {
  min-height: 100vh;
  background-color: #f3f4f6;
  font-family: 'Segoe UI', system-ui, sans-serif;
  color: #1f2937;
  padding: 2rem;
}

.header {
  display: flex;
  align-items: center;
  gap: 1rem;
  margin-bottom: 2rem;
  padding-bottom: 1rem;
  border-bottom: 2px solid #e5e7eb;
}

.logo {
  font-size: 2.5rem;
}

.title-area h1 {
  margin: 0;
  font-size: 1.8rem;
  color: #111827;
}

.subtitle {
  margin: 0;
  color: #6b7280;
  font-size: 0.9rem;
}

.main-content {
  max-width: 700px;
  margin: 0 auto;
}

.card {
  background: white;
  border-radius: 12px;
  box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1), 0 2px 4px -1px rgba(0, 0, 0, 0.06);
  overflow: hidden;
}

.card-header {
  background-color: #f9fafb;
  padding: 1.5rem;
  border-bottom: 1px solid #e5e7eb;
}

.card-header h2 {
  margin: 0 0 0.5rem 0;
  font-size: 1.25rem;
  color: #374151;
}

.card-header p {
  margin: 0;
  color: #6b7280;
  font-size: 0.9rem;
}

.card-body {
  padding: 1.5rem;
}

.form-group {
  margin-bottom: 1.5rem;
  display: flex;
  flex-direction: column;
}

.form-group label {
  font-weight: 600;
  margin-bottom: 0.5rem;
  color: #374151;
  font-size: 0.95rem;
}

.form-group input {
  padding: 0.75rem 1rem;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  font-size: 1rem;
  transition: all 0.2s;
}

.form-group input:focus {
  outline: none;
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.2);
}

.hint {
  font-size: 0.8rem;
  color: #9ca3af;
  margin-top: 0.4rem;
}

.primary-btn {
  width: 100%;
  padding: 0.875rem;
  background-color: #2563eb;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 1.1rem;
  font-weight: 600;
  cursor: pointer;
  transition: background-color 0.2s;
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 0.5rem;
}

.primary-btn:hover:not(:disabled) {
  background-color: #1d4ed8;
}

.primary-btn:disabled {
  background-color: #9ca3af;
  cursor: not-allowed;
}

.spinner {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  100% { transform: rotate(360deg); }
}

.result-box {
  margin-top: 1.5rem;
  padding: 1rem;
  border-radius: 8px;
  font-family: monospace;
  font-size: 0.9rem;
  overflow-x: auto;
}

.result-box pre {
  margin: 0;
  white-space: pre-wrap;
}

.success {
  background-color: #f0fdf4;
  border: 1px solid #bbf7d0;
  color: #166534;
}

.error {
  background-color: #fef2f2;
  border: 1px solid #fecaca;
  color: #991b1b;
}
</style>