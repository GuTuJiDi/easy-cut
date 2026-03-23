<template>
  <div class="container">
    <h1>易剪 (EasyCut) V1.0 </h1>
    <div class="section">
      <button @click="testEngine">1. 检测底层切割引擎</button>
      <p v-if="engineStatus" class="status">{{ engineStatus }}</p>
    </div>
    <hr>
    <div class="section">
      <h3>2. 无损视频分割</h3>
      <div class="form-group">
        <label>输入视频绝对路径:</label>
        <input v-model="splitParams.input" type="text" placeholder="例如: D:\test.mp4" />
      </div>
      <div class="form-group">
        <label>输出视频绝对路径:</label>
        <input v-model="splitParams.output" type="text" placeholder="例如: D:\output.mp4" />
      </div>
      <div class="form-group row">
        <div>
          <label>起始时间 (秒):</label>
          <input v-model="splitParams.startTime" type="text" placeholder="0" />
        </div>
        <div>
          <label>截取时长 (秒):</label>
          <input v-model="splitParams.duration" type="text" placeholder="15" />
        </div>
      </div>

      <button @click="runSplitTask" :disabled="isProcessing">
        {{ isProcessing ? '处理中...' : '执行分割' }}
      </button>
      <p v-if="splitResult" :class="{status: true, error: isError ,success: !isError}">
        {{ splitResult }}
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
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
// Charlie 新增的前端逻辑
const isProcessing = ref(false);
const splitResult = ref('');
const isError = ref(false);

const splitParams = reactive({
  input: '',
  output: '',
  startTime: '0',
  duration: '15'
});

async function runSplitTask() {
  if (!splitParams.input || !splitParams.output) {
    splitResult.value = "请填写完整的路径！";
    isError.value = true;
    return;
  }
  isProcessing.value = true;
  splitResult.value = '';
  isError.value = false;

  try {
    // 调用 Bob 开发的接口
    const result = await invoke<string>('split_video', {
      inputPath: splitParams.input,
      outputPath: splitParams.output,
      startTime: splitParams.startTime,
      duration: splitParams.duration
    });
    splitResult.value = `✅ ${result}`;
  } catch (error) {
    splitResult.value = `❌ ${error}`;
    isError.value = true;
  } finally {
    isProcessing.value = false;
  }
}
</script>

<style scoped>
/* 简单的样式保持不变 */
.container { padding: 20px; font-family: sans-serif; max-width: 600px; margin: 0 auto; }
.section { margin-bottom: 20px; }
.form-group { display: flex; flex-direction: column; margin-bottom: 10px; }
.form-group.row { flex-direction: row; gap: 10px; }
.form-group.row > div { flex: 1; display: flex; flex-direction: column; }
input { padding: 6px; margin-top: 4px; border: 1px solid #ccc; border-radius: 4px; }
button { padding: 8px 16px; background-color: #2563eb; color: white; border: none; border-radius: 4px; cursor: pointer; }
button:disabled { background-color: #9ca3af; }
hr { margin: 20px 0; border-top: 1px solid #eee; }
.status { margin-top: 10px; padding: 10px; border-radius: 4px; background: #f0f0f0;}
.success { background-color: #dcfce7; color: #166534; }
.error { background-color: #fee2e2; color: #991b1b; }
</style>