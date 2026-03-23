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
            <label>🎬 输入视频文件</label>
            <div class="input-with-btn">
              <input v-model="batchParams.inputPath" type="text" readonly placeholder="请选择需要分割的视频..." />
              <button class="secondary-btn" @click="selectInputFile">浏览文件</button>
            </div>
          </div>

          <div class="form-group">
            <label>📁 输出保存目录</label>
            <div class="input-with-btn">
              <input v-model="batchParams.outputDir" type="text" readonly placeholder="请选择保存目录..." />
              <button class="secondary-btn" @click="selectOutputDir">更改目录</button>
            </div>
            <span class="hint" v-if="batchParams.videoName && batchParams.outputDir">
              预期生成: {{ batchParams.outputDir }}\<b>{{ batchParams.videoName }}</b>\{{ batchParams.videoName }}_part1.mp4
            </span>
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

          <div v-if="resultLogs.length > 0" :class="['result-box', isError ? 'error' : 'success']">
            <div class="result-header">
              <span>🎉 批量分割任务完成！(共 {{ resultLogs.length }} 段)</span>
              <button class="text-btn" @click="isExpanded = !isExpanded" v-if="resultLogs.length > 5">
                {{ isExpanded ? '收起列表' : '展开全部' }}
              </button>
            </div>
            <ul class="log-list">
              <li v-for="(log, index) in visibleLogs" :key="index">{{ log }}</li>
              <li v-if="!isExpanded && resultLogs.length > 5" class="ellipsis">... 还有 {{ resultLogs.length - 5 }} 个片段被隐藏 ...</li>
            </ul>
          </div>
        </div>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import {ref, reactive, computed} from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog'; // <--- 引入 dialog 插件
// 状态管理
const isProcessing = ref(false);
const resultLogs = ref<string[]>([]); // 改为数组，方便渲染折叠列表
const isError = ref(false);
const isExpanded = ref(false); // 控制是否展开长列表
// 表单数据绑定
const batchParams = reactive({
  inputPath: '',
  outputDir: '',
  videoName: '', // 纯视频名称（如：20231024.先导片）
  segmentDuration: 300
});
const visibleLogs = computed(() => {
  return isExpanded.value ? resultLogs.value : resultLogs.value.slice(0, 5);
});
async function selectInputFile() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'Video Files', extensions: ['mp4', 'mkv', 'mov'] }]
  });

  if (selected && typeof selected === 'string') {
    batchParams.inputPath = selected;

    // 智能解析路径 (兼容 Windows \ 和 Mac/Linux /)
    const normalizedPath = selected.replace(/\\/g, '/');
    const pathParts = normalizedPath.split('/');
    const fileNameWithExt = pathParts.pop() || '';

    // 提取纯名称 (去掉 .mp4)
    const dotIndex = fileNameWithExt.lastIndexOf('.');
    batchParams.videoName = dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt;

    // 默认输出目录设为原视频所在目录
    batchParams.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');
  }
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') {
    batchParams.outputDir = selected;
  }
}
// 触发后端切割任务
async function runBatchSplit() {
  if (!batchParams.inputPath || !batchParams.outputDir) return;
  isProcessing.value = true;
  resultLogs.value = [];
  isError.value = false;
  isExpanded.value = false; // 每次执行重置折叠状态

  try {
    const result = await invoke<string>('batch_split_by_duration', {
      inputPath: batchParams.inputPath,
      outputDir: batchParams.outputDir,
      videoName: batchParams.videoName,
      segmentDuration: batchParams.segmentDuration
    });
    // 按行拆分后端返回的日志
    resultLogs.value = result.split('\n');
  } catch (error) {
    resultLogs.value = [`❌ 发生错误: ${error}`];
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

/* 在之前的样式底部追加 */
.input-with-btn {
  display: flex;
  gap: 10px;
}
.input-with-btn input {
  flex: 1;
  background-color: #f9fafb; /* 只读状态的底色 */
  cursor: not-allowed;
}
.secondary-btn {
  padding: 0 1rem;
  background-color: #e5e7eb;
  color: #374151;
  border: 1px solid #d1d5db;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 500;
  transition: all 0.2s;
  white-space: nowrap;
}
.secondary-btn:hover {
  background-color: #d1d5db;
}

/* 补充折叠列表的样式 */
.result-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.5rem;
  font-weight: bold;
}
.text-btn {
  background: none;
  border: none;
  color: #2563eb;
  cursor: pointer;
  font-size: 0.85rem;
  padding: 0;
}
.text-btn:hover { text-decoration: underline; }
.log-list {
  list-style: none;
  padding: 0;
  margin: 0;
  font-family: monospace;
  font-size: 0.85rem;
  line-height: 1.5;
}
.ellipsis {
  color: #6b7280;
  font-style: italic;
  margin-top: 0.5rem;
}
</style>