<template>
  <div class="view-container">
    <header class="view-header">
      <h1 class="view-title">视频格式转换 <span class="pro-badge-title">PRO</span></h1>
      <p class="view-subtitle">采用高保真转码引擎，秒级转换视频封装格式，兼容绝大多数播放器。</p>
    </header>

    <div class="workspace-grid">
      <div class="card form-card">
        <div class="form-scroller hide-scrollbar">
          <div class="card-body">
            <div class="section-title">1. 选择源视频</div>
            <div class="form-group source-group">
              <button v-if="!params.inputPath" class="upload-dropzone-btn pro-theme" @click="selectInputFile">
                <span class="upload-icon">✨</span>
                <span class="upload-text">点击选择待转换视频</span>
              </button>

              <div v-else class="active-file-wrapper">
                <div class="active-file-badge">
                  <span class="file-icon">🎬</span>
                  <span class="file-name" :title="params.inputPath">{{ videoFileName }}</span>
                  <div class="badge-actions">
                    <button class="icon-btn" @click="selectInputFile" title="更换视频">🔄</button>
                    <button class="icon-btn" @click="clearInputFile" title="移除视频">✖</button>
                  </div>
                </div>
              </div>
            </div>

            <hr class="divider" />

            <div class="section-title">2. 转换设置</div>
            <div class="form-group">
              <label>🎯 目标格式</label>
              <div class="format-grid">
                <button v-for="fmt in formats" :key="fmt"
                        class="format-btn"
                        :class="{'active': targetFormat === fmt}"
                        @click="targetFormat = fmt">
                  .{{ fmt.toUpperCase() }}
                </button>
              </div>
            </div>

            <div class="form-group">
              <label>📁 保存目录</label>
              <div class="input-with-btn">
                <input v-model="params.outputDir" type="text" readonly class="file-input dir-input" />
                <button class="secondary-btn" @click="selectOutputDir">更改</button>
              </div>
            </div>

            <div class="path-preview-card" v-if="isReady">
              <div class="preview-header">📂 预期生成文件</div>
              <div class="preview-path" :title="`${params.outputDir}\\${params.outputName}`">
                {{ params.outputDir }}\<strong class="highlight-dir">{{ params.outputName }}</strong>
              </div>
            </div>

          </div>
        </div>

        <div class="form-footer action-footer">
          <button
              v-if="!isProcessing"
              class="primary-btn execute-btn pro-execute"
              @click="runConvert"  :disabled="!isReady"
          >
            🚀 开始无损转换 </button>

          <button
              v-else
              class="execute-btn danger-execute"
              @click="cancelTask"
          >
            <span class="spinner">⚙️</span>
            🛑 停止当前任务
          </button>
        </div>
      </div>

      <div class="card result-card">
        <div v-if="!resultLog && !isProcessing" class="empty-state">
          <span class="empty-icon">🔄</span>
          <p>转换结果将在此展示</p>
        </div>

        <div v-else class="result-content" :class="{ 'has-error': isError }">
          <div class="result-header">
            <div class="status-title">
              <span v-if="isProcessing">⏳ 正在转码 (取决于文件大小)...</span>
              <span v-else-if="isError">❌ 转换失败</span>
              <span v-else>🎉 转换完成</span>
            </div>
            <button v-if="!isProcessing && !isError && resultLog" class="secondary-btn small" @click="openTargetFolder">
              📂 打开目录
            </button>
          </div>

          <div class="error-msg hide-scrollbar" v-if="isError">{{ resultLog }}</div>

          <div class="success-box" v-else-if="!isProcessing && resultLog">
            <div class="log-info">
              <span class="check-icon">✓</span>
              <span class="log-filename" :title="resultLog">{{ extractedFileName }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useRoute } from 'vue-router'; // 🌟 引入路由
import { useAuthStore } from '../stores/auth';

const authStore = useAuthStore();
const route = useRoute(); // 🌟 初始化路由

const formats = ['mp4', 'mkv', 'mov', 'avi'];
const targetFormat = ref('mp4');

const isProcessing = ref(false);
const isError = ref(false);
const resultLog = ref('');

const params = ref({
  inputPath: '',
  outputDir: '',
  outputName: ''
});

// 🌟 核心定制：专属格式转换页面的跨路由监听
watch(
    () => route.query.loadVideo,
    (newPath) => {
      // 领地防线：确保只在格式转换路由下生效（兼容名为 format 或 transform 的路由）
      if (!route.path.includes('format') && !route.path.includes('transform')) return;

      if (newPath && typeof newPath === 'string' && params.value.inputPath !== newPath) {
        params.value.inputPath = newPath;
        const pathParts = newPath.replace(/\\/g, '/').split('/');
        pathParts.pop(); // 移除文件名，保留目录
        params.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

        resultLog.value = '';
        // 注意：这里不需要手动设置 params.value.outputName！
        // 因为下面的 watch([videoFileName, targetFormat]) 会自动侦听到 videoFileName 的变化，
        // 并根据当前的 targetFormat 自动无缝生成输出文件名。
      }
    },
    { immediate: true }
);

const videoFileName = computed(() => params.value.inputPath.split(/[/\\]/).pop() || '');
const extractedFileName = computed(() => resultLog.value.replace('✅ 生成: ', '').split(/[/\\]/).pop() || '');
const isReady = computed(() => params.value.inputPath && params.value.outputDir && params.value.outputName);

// 监听格式变化，自动更新输出文件名
watch([videoFileName, targetFormat], () => {
  if (!videoFileName.value) return;
  const dotIndex = videoFileName.value.lastIndexOf('.');
  const baseName = dotIndex > -1 ? videoFileName.value.substring(0, dotIndex) : videoFileName.value;
  params.value.outputName = `${baseName}_转码.${targetFormat.value}`;
});
// 🌟 在两个组件的 <script setup> 中添加取消方法
async function cancelTask() {
  try {
    // 调用我们在 main.rs 中注册的取消指令
    await invoke('cancel_active_tasks');
    // 注意：这里不需要手动把 isProcessing 设为 false。
    // 后端被掐断后，原本的 runExtract/runConvert 会抛出错误，
    // 在 catch 块中会自动处理状态。
  } catch (e) {
    console.error("发送取消指令失败:", e);
  }
}
async function selectInputFile() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi', 'flv', 'wmv'] }] });
  if (selected && typeof selected === 'string') {
    params.value.inputPath = selected;
    const pathParts = selected.replace(/\\/g, '/').split('/');
    pathParts.pop(); // remove file name
    params.value.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');
    resultLog.value = '';
  }
}

function clearInputFile() {
  params.value.inputPath = ''; params.value.outputDir = ''; params.value.outputName = ''; resultLog.value = '';
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') params.value.outputDir = selected;
}

async function runConvert() {
  if (!authStore.isPro) {
    alert("🔒 该功能为 PRO 专业版独占，请先在全局设置中激活！");
    return;
  }

  isProcessing.value = true;
  isError.value = false;
  resultLog.value = '';

  try {
    const res = await invoke<string>('execute_format_convert', {
      params: {
        input_path: params.value.inputPath,
        output_dir: params.value.outputDir,
        output_name: params.value.outputName,
        // license_str: authStore.licenseKey // 🛡️ 商业大闸
      },
      sessionToken: authStore.sessionToken,
    });
    resultLog.value = res;
  } catch (error) {
    const errMsg = String(error);
    if (errMsg.includes('主动取消')) {
      // 用户主动取消，不算错误
      resultLog.value = '🛑 任务已取消';
      isError.value = false;
    } else {
      // 真正的报错
      resultLog.value = errMsg;
      isError.value = true;
      if (errMsg.includes("拦截")) alert(errMsg);
    }
  } finally {
    isProcessing.value = false;
  }
}

async function openTargetFolder() {
  try { await invoke('open_folder', { path: params.value.outputDir }); } catch (e) { console.error(e); }
}
</script>

<style scoped>
/* 保持高度一致的 UI 组件库样式 */
.hide-scrollbar { -ms-overflow-style: none; scrollbar-width: none; }
.hide-scrollbar::-webkit-scrollbar { display: none; }
.view-container { display: flex; flex-direction: column; height: 100vh; overflow: hidden; padding: 1.2rem; box-sizing: border-box; background: #f8fafc;}
.view-header { margin-bottom: 0.8rem; }
.view-title { margin: 0 0 0.4rem 0; display: flex; align-items: center; gap: 10px;}
.view-subtitle { margin: 0; color: #6b7280; font-size: 0.9rem; }
.pro-badge-title { background: linear-gradient(135deg, #f59e0b, #d97706); color: white; font-size: 0.8rem; padding: 4px 8px; border-radius: 6px; font-weight: 900;}

.workspace-grid { display: grid; grid-template-columns: minmax(320px, 450px) 1fr; gap: 1.2rem; flex: 1; min-height: 0; }
.card { display: flex; flex-direction: column; background: white; border-radius: 12px; box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.05); border: 1px solid #e5e7eb; overflow: hidden; }
.form-scroller { flex: 1; min-height: 0; overflow-y: auto; }
.card-body { padding: 1.2rem 1.2rem 0.5rem 1.2rem; }
.form-footer { padding: 0.8rem 1.2rem; background: white; border-top: 1px solid #e5e7eb; flex-shrink: 0; }

.section-title { font-size: 1.05rem; font-weight: 600; margin-bottom: 0.8rem; color: #111827;}
.form-group { margin-bottom: 1rem; display: flex; flex-direction: column; }
.form-group label { font-size: 0.85rem; color: #4b5563; margin-bottom: 0.4rem; font-weight: 600;}

.upload-dropzone-btn { width: 100%; min-height: 116px; background: #f8fafc; border: 2px dashed #cbd5e1; border-radius: 8px; cursor: pointer; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 8px; transition: all 0.2s ease;}
.upload-dropzone-btn.pro-theme:hover { background: #fffbeb; border-color: #f59e0b; }
.upload-dropzone-btn.pro-theme .upload-text { color: #d97706; font-weight: 700;}

.active-file-wrapper { display: flex; flex-direction: column; min-height: 116px;}
.active-file-badge { display: flex; align-items: center; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 8px; gap: 0.75rem; border: 1px solid #cbd5e1;}
.file-icon { font-size: 1.4rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.9rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: monospace;}
.badge-actions { display: flex; gap: 0.25rem; border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}
.icon-btn { background: transparent; border: none; color: #64748b; cursor: pointer; transition: 0.2s;}

.input-with-btn { display: flex; gap: 8px; }
.file-input { flex: 1; padding: 0.6rem 1rem; border: 1px solid #d1d5db; border-radius: 8px; background: #f9fafb; color: #4b5563; outline: none; font-family: monospace;}

/* 专属：格式选择网格 */
.format-grid { display: grid; grid-template-columns: repeat(4, 1fr); gap: 10px; }
.format-btn { padding: 10px; border: 1px solid #d1d5db; background: white; border-radius: 8px; font-weight: bold; color: #475569; cursor: pointer; transition: 0.2s; font-family: monospace; font-size: 1rem;}
.format-btn.active { border-color: #f59e0b; background: #fffbeb; color: #d97706; box-shadow: 0 2px 4px rgba(245, 158, 11, 0.1);}

.path-preview-card { margin-top: 0.75rem; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden; }
.preview-header { font-size: 0.75rem; background: #e2e8f0; color: #475569; padding: 4px 8px; font-weight: bold; }
.preview-path { padding: 8px; font-family: 'Consolas', monospace; font-size: 0.8rem; color: #64748b; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; direction: rtl; text-align: left; }
.highlight-dir { color: #f59e0b; font-weight: bold; }

.execute-btn { width: 100%; padding: 0.8rem; font-size: 1.05rem; justify-content: center; border: none; border-radius: 8px; color: white; font-weight: bold; cursor: pointer; transition: 0.2s;}
.pro-execute { background: linear-gradient(135deg, #f59e0b, #d97706); }
.pro-execute:hover:not(:disabled) { box-shadow: 0 4px 12px rgba(217, 119, 6, 0.3); transform: translateY(-1px);}
.pro-execute:disabled { background: #cbd5e1; cursor: not-allowed;}

.result-card { display: flex; flex-direction: column;}
.empty-state { flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #94a3b8; }
.empty-icon { font-size: 3rem; margin-bottom: 0.5rem; opacity: 0.5;}
.result-content { padding: 1.5rem; display: flex; flex-direction: column;}
.result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.75rem;}
.status-title { font-weight: 700; color: #166534; font-size: 1.1rem;}
.has-error .status-title { color: #dc2626; }
.error-msg { color: #dc2626; font-family: monospace; white-space: pre-wrap; background: #fef2f2; padding: 1rem; border-radius: 8px; border: 1px solid #fca5a5;}

.success-box { background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 15px;}
.log-info { display: flex; align-items: center; gap: 8px;}
.check-icon { color: #10b981; font-weight: bold;}
.log-filename { font-family: monospace; font-size: 0.9rem; color: #334155; font-weight: 600;}
.secondary-btn { padding: 0.5rem 1rem; background: white; border: 1px solid #d1d5db; border-radius: 6px; cursor: pointer; font-weight: 600;}
.secondary-btn:hover { background: #f8fafc; }

.action-footer { display: flex; gap: 10px; }
.danger-execute {
  background: linear-gradient(135deg, #ef4444, #dc2626);
  display: flex;
  align-items: center;
  gap: 8px;
}
.danger-execute:hover {
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);
  transform: translateY(-1px);
}
/* 让齿轮图标转起来 */
.spinner { animation: spin 2s linear infinite; }
@keyframes spin { 100% { transform: rotate(360deg); } }
</style>