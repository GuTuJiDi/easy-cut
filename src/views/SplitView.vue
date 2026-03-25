<template>
  <div class="view-container">

    <header class="view-header">
      <h1 class="view-title">基础无损分割</h1>
      <p class="view-subtitle">极速流复制技术，无需重新编码，瞬间完成视频切片。</p>
    </header>

    <div class="tabs-header">
      <div class="tabs-nav">
        <button class="tab-btn active"><span class="icon">⏱️</span> 固定时长分割</button>
        <button class="tab-btn disabled" title="Pro 版本功能，即将推出"><span class="icon">🔢</span> 固定数量分割 <span class="pro-tag">PRO</span></button>
      </div>
    </div>

    <div class="workspace-grid">
      <!-- ================= 左侧：表单配置区 ================= -->
      <div class="card form-card">
        <div class="card-body">
          <div class="section-title">1. 选择视频源</div>

          <div class="form-group">
            <!-- 🌟 修复 1：引入绝美的胶囊徽章替换丑陋的 input -->
            <button v-if="!batchParams.inputPath" class="primary-btn outline-dash" @click="selectInputFile">
              <span class="icon">📁</span> 点击选择待分割视频
            </button>

            <div v-else class="active-file-badge">
              <span class="file-icon">🎬</span>
              <span class="file-name" :title="batchParams.inputPath">{{ videoFileName }}</span>
              <div class="badge-actions">
                <button class="icon-btn" @click="selectInputFile" title="更换视频">🔄</button>
                <button class="icon-btn" @click="clearInputFile" title="移除视频">✖</button>
              </div>
            </div>

            <transition name="fade">
              <div class="duration-badge" v-if="videoDurationSec > 0">
                <span class="icon">⏱️</span> 视频总长:
                <span class="time-val">{{ showDurationInMinutes ? (videoDurationSec / 60).toFixed(2) : videoDurationSec }}</span>
                <button class="text-link" @click="showDurationInMinutes = !showDurationInMinutes">
                  切换为{{ showDurationInMinutes ? '秒' : '分钟' }}
                </button>
              </div>
            </transition>
          </div>

          <hr class="divider" />

          <div class="section-title">2. 输出设置</div>
          <div class="form-group">
            <label>📁 目标保存目录</label>
            <div class="input-with-btn">
              <!-- 为了让目录更直观，这里保留 input，但优化了样式 -->
              <input v-model="batchParams.outputDir" type="text" readonly placeholder="默认保存在原视频同级目录..." class="file-input dir-input" />
              <button class="secondary-btn" @click="selectOutputDir">更改</button>
            </div>

            <!-- 🌟 修复 2：绝美的单行截断式预期路径卡片 -->
            <div class="path-preview-card" v-if="batchParams.videoName && batchParams.outputDir">
              <div class="preview-header">📂 预期生成示例</div>
              <div class="preview-path" :title="`${batchParams.outputDir}\\${batchParams.videoName}\\[01]_${batchParams.videoName}.mp4`">
                {{ batchParams.outputDir }}\<strong class="highlight-dir">{{ batchParams.videoName }}</strong>\[01]_{{ batchParams.videoName }}.mp4
              </div>
            </div>
          </div>

          <div class="form-group">
            <label>⏳ 每个片段时长</label>
            <div class="duration-input-wrapper">
              <!-- 🌟 修复 3：复合型输入框，支持毫秒(0.1)与单位一键切换 -->
              <div class="input-with-addon">
                <input v-model.number="displayDuration" type="number" step="0.1" min="0.1" class="number-input" />
                <button class="addon-btn" @click="toggleDurationUnit">
                  {{ isDurationInMinutes ? '分钟' : '秒' }} 🔃
                </button>
              </div>
              <span class="duration-hint">
                {{ isDurationInMinutes ? `≈ ${displayDuration * 60} 秒/段` : `≈ ${(displayDuration / 60).toFixed(2)} 分钟/段` }}
              </span>
            </div>
          </div>

          <!-- 🌟 修复 4：文案修正 -->
          <button class="primary-btn mt-4" @click="runBatchSplit" :disabled="!isReadyToProcess || isProcessing">
            <span v-if="isProcessing" class="spinner">⚙️</span>
            {{ isProcessing ? '全速引擎切割中...' : '🚀 开始极速分割' }}
          </button>
        </div>
      </div>

      <!-- ================= 右侧：结果列表区 (第一阶段暂保原样) ================= -->
      <!-- ================= 右侧：结果列表区 (V2.0 豪华版) ================= -->
      <div class="card result-card">
        <div v-if="resultLogs.length === 0 && !isProcessing" class="empty-state">
          <span class="empty-icon">✂️</span>
          <p>分割任务列表</p>
          <span class="hint">左侧点击执行后，此处将展示生成结果</span>
        </div>

        <div v-else class="result-content" :class="{ 'has-error': isError }">

          <div class="result-header">
            <div class="status-title">
              <span v-if="isProcessing">⏳ 全速切割中...</span>
              <span v-else-if="isError">❌ 处理遇到错误</span>
              <div v-else class="success-header-group">
                <span>🎉 任务完成 (共 {{ parsedLogs.length }} 段)</span>
                <!-- 🌟 修复：把耗时作为荣誉徽章挂在标题旁边 -->
                <transition name="pop-in">
                  <span v-if="splitCostTime" class="time-badge">
                    ⏱️ 耗时: <strong>{{ splitCostTime }}</strong> 秒
                  </span>
                </transition>
              </div>
            </div>

            <button v-if="!isProcessing && !isError && parsedLogs.length > 0" class="secondary-btn small" @click="openTargetFolder">
              📂 打开输出目录
            </button>
          </div>

          <!-- 错误展示 -->
          <div class="error-msg" v-if="isError">{{ resultLogs[0] }}</div>

          <!-- 🌟 全新解析后的交互式列表 -->
          <div class="list-container" v-else>
            <ul class="parsed-log-list">
              <li v-for="(item, index) in visibleLogs" :key="index" class="log-item">
                <div class="log-info">
                  <span class="check-icon">✓</span>
                  <span class="log-filename" :title="item.path">{{ item.filename }}</span>
                  <!-- 🌟 新增：优雅的时长小徽章 -->
                  <span class="log-duration">{{ item.durationStr }}</span>
                </div>

                <!-- 🌟 升级的悬停操作栏 -->
                <div class="log-actions">
                  <!-- 1. 继续分割按钮 -->
                  <button class="action-btn split-btn" @click="continueSplit(item.path)" title="将此片段作为源视频，继续细分">
                    <span class="icon">✂️</span> 继续分割
                  </button>

                  <!-- 2. 打轴标记按钮 -->
                  <button class="action-btn marker-btn" @click="goToMarker(item.path)" title="前往打轴工作流">
                    <span class="icon">🏷️</span> 打轴标记
                  </button>

                  <!-- 3. 原有的播放按钮 -->
                  <button class="action-icon play-btn" @click="playVideoWithSystem(item.path)" title="调用系统播放器预览">
                    ▶
                  </button>
                </div>
              </li>
            </ul>

            <!-- 🌟 手风琴展开/收起按钮 -->
            <div class="list-toggle" v-if="parsedLogs.length > 6">
              <button class="toggle-btn" @click="isListExpanded = !isListExpanded">
                {{ isListExpanded ? '▲ 收起列表' : `▼ 展开其余 ${parsedLogs.length - 6} 个片段` }}
              </button>
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useRouter } from 'vue-router'; // 🌟 新增：引入路由
const router = useRouter(); // 初始化路由


const isProcessing = ref(false);
const resultLogs = ref<string[]>([]);
const isError = ref(false);

const videoDurationSec = ref(0);
const showDurationInMinutes = ref(true);

const batchParams = reactive({
  inputPath: '', outputDir: '', videoName: '', segmentDuration: 300 // 后端始终接收秒
});

// 🌟 修复 3：前端展示用的时长状态
const displayDuration = ref(5); // 默认 5 分钟
const isDurationInMinutes = ref(true);
// --- 新增状态区 ---
const splitCostTime = ref(''); // 存储分割总耗时
const isListExpanded = ref(false); // 控制列表展开/收起

// --- 🌟 核心：将长字符串日志，动态解析为结构化数据 ---
const parsedLogs = computed(() => {
  if (resultLogs.value.length === 0 || isError.value) return [];

  return resultLogs.value.map((log, index) => {
    const cleanPath = log.replace('✅ 生成: ', '').trim();
    const filename = cleanPath.split(/[/\\]/).pop() || cleanPath;

    // 💡 智能模拟时长：在后端真正返回每个片段时长前，我们在前端进行合理的推算
    let segDur = batchParams.segmentDuration;
    // 如果是最后一个片段，计算余数时长
    if (index === resultLogs.value.length - 1 && videoDurationSec.value > 0) {
      const remainder = videoDurationSec.value % batchParams.segmentDuration;
      if (remainder > 0) segDur = remainder;
    }
    const durationStr = formatDuration(segDur);

    return { filename, path: cleanPath, durationStr };
  });
});

// --- 🌟 核心：控制可视列表（手风琴效果） ---
const visibleLogs = computed(() => {
  if (isListExpanded.value) return parsedLogs.value;
  return parsedLogs.value.slice(0, 6); // 默认只展示前 6 条
});
// 监听前端显示变化，自动换算成秒给后端
watch([displayDuration, isDurationInMinutes], () => {
  if (displayDuration.value > 0) {
    batchParams.segmentDuration = isDurationInMinutes.value ? displayDuration.value * 60 : displayDuration.value;
  }
});

function toggleDurationUnit() {
  isDurationInMinutes.value = !isDurationInMinutes.value;
  // 切换时自动换算数值，保证总时间不变
  if (isDurationInMinutes.value) {
    displayDuration.value = Number((displayDuration.value / 60).toFixed(2));
  } else {
    displayDuration.value = Math.round(displayDuration.value * 60);
  }
}

// 🌟 修复 1：提取纯文件名
const videoFileName = computed(() => {
  if (!batchParams.inputPath) return '';
  return batchParams.inputPath.split(/[/\\]/).pop() || '';
});

const isReadyToProcess = computed(() => batchParams.inputPath && batchParams.outputDir && batchParams.segmentDuration > 0);

async function selectInputFile() {
  const selected = await open({ multiple: false, filters: [{ name: 'Videos', extensions: ['mp4', 'mkv', 'mov', 'avi'] }] });
  if (selected && typeof selected === 'string') {
    batchParams.inputPath = selected;
    const pathParts = selected.replace(/\\/g, '/').split('/');
    const fileNameWithExt = pathParts.pop() || '';
    const dotIndex = fileNameWithExt.lastIndexOf('.');
    batchParams.videoName = dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt;
    batchParams.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

    try {
      videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: selected });
    } catch (e) { console.error("无法获取时长", e); }
    resultLogs.value = [];
  }
}

function clearInputFile() {
  batchParams.inputPath = ''; batchParams.outputDir = ''; batchParams.videoName = '';
  videoDurationSec.value = 0; resultLogs.value = [];
}

async function selectOutputDir() {
  const selected = await open({ directory: true, multiple: false });
  if (selected && typeof selected === 'string') batchParams.outputDir = selected;
}

async function openTargetFolder() {
  const targetPath = `${batchParams.outputDir}\\${batchParams.videoName}`;
  try { await invoke('open_folder', { path: targetPath }); } catch (e) { console.error(e); }
}

async function runBatchSplit() {
  if (!isReadyToProcess.value) return;
  isProcessing.value = true; resultLogs.value = []; isError.value = false;
  splitCostTime.value = ''; // 清空耗时
  isListExpanded.value = false; // 默认收起列表
  const startTime = performance.now(); // ⏱️ 开始计时
  try {
    const result = await invoke<string>('batch_split_by_duration', batchParams);
    resultLogs.value = result.split('\n').filter(line => line.trim() !== '');
    // ⏱️ 计算耗时
    splitCostTime.value = ((performance.now() - startTime) / 1000).toFixed(1);
  } catch (error) {
    resultLogs.value = [`${error}`]; isError.value = true;
  } finally {
    isProcessing.value = false;
  }
}

// --- 🌟 新增：调用系统默认播放器播放 ---
async function playVideoWithSystem(filePath: string) {
  try {
    await invoke('open_file', { path: filePath });
  } catch (e) {
    console.error("播放失败", e);
  }
}

// 🌟 核心升级：为解析日志加入“时长模拟”与格式化
function formatDuration(seconds: number): string {
  if (!seconds || isNaN(seconds)) return '00:00';
  const m = Math.floor(seconds / 60).toString().padStart(2, '0');
  const s = Math.floor(seconds % 60).toString().padStart(2, '0');
  return `${m}:${s}`;
}


// 🌟 新增交互：将片段作为源视频继续分割
async function continueSplit(targetPath: string) {
  batchParams.inputPath = targetPath;
  const pathParts = targetPath.replace(/\\/g, '/').split('/');
  const fileNameWithExt = pathParts.pop() || '';
  const dotIndex = fileNameWithExt.lastIndexOf('.');

  batchParams.videoName = dotIndex > -1 ? fileNameWithExt.substring(0, dotIndex) : fileNameWithExt;
  batchParams.outputDir = pathParts.join(navigator.platform.includes('Win') ? '\\' : '/');

  // 获取新片段的时长
  try {
    videoDurationSec.value = await invoke<number>('get_video_duration_cmd', { videoPath: targetPath });
  } catch (e) { console.error("无法获取时长", e); }

  // 清空结果列表，收起界面，准备下一次切割
  resultLogs.value = [];
  isListExpanded.value = false;
  splitCostTime.value = '';
}

// 🌟 新增交互：跳转到打轴标记页面，并传递视频路径
function goToMarker(targetPath: string) {
  // 利用 Vue Router 的 query 参数，把路径传给智能打轴页面
  router.push({ path: '/marker', query: { loadVideo: targetPath } });
}
</script>

<style scoped>
/* 网格布局 */
.workspace-grid { display: grid; grid-template-columns: minmax(320px, 450px) 1fr; gap: 1.5rem; flex: 1; min-height: 0; }
.form-card .card-body { padding: 1.5rem; }
.section-title { font-size: 1.05rem; font-weight: 600; margin-bottom: 1rem; color: #111827;}
.form-group { margin-bottom: 1.25rem; display: flex; flex-direction: column; }
.form-group label { font-size: 0.85rem; color: #4b5563; margin-bottom: 0.4rem; font-weight: 500;}
.mt-4 { margin-top: 1.5rem; }
/* 🌟 右侧 V2.0 豪华版交互列表样式 */
.success-header-group { display: flex; align-items: center; gap: 0.75rem; }
.time-badge { background: #ecfdf5; color: #059669; padding: 2px 10px; border-radius: 20px; font-size: 0.8rem; font-weight: 500; border: 1px solid #a7f3d0; display: inline-flex; align-items: center; gap: 4px; }
.time-badge strong { font-size: 0.95rem; font-family: 'Consolas', monospace; font-weight: 800; }

.list-container { display: flex; flex-direction: column; flex: 1; overflow: hidden; }
.parsed-log-list { flex: 1; overflow-y: auto; list-style: none; padding: 0; margin: 0; padding-right: 5px;}
.log-item { display: flex; justify-content: space-between; align-items: center; padding: 0.6rem 0.5rem; border-bottom: 1px solid #f1f5f9; transition: all 0.2s; border-radius: 6px;}
.log-item:hover { background: #f8fafc; border-color: transparent; box-shadow: 0 1px 3px rgba(0,0,0,0.05); }

.log-info { display: flex; align-items: center; gap: 8px; flex: 1; min-width: 0; }
.log-filename { font-family: 'Consolas', monospace; font-size: 0.85rem; color: #334155; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-weight: 600; cursor: default;}

.log-actions { display: flex; gap: 4px; opacity: 0; transition: opacity 0.2s; }
.log-item:hover .log-actions { opacity: 1; }

.action-icon { background: white; border: 1px solid #e2e8f0; color: #64748b; font-size: 0.9rem; padding: 4px 8px; border-radius: 6px; cursor: pointer; transition: all 0.2s; display: flex; align-items: center; justify-content: center; }
.action-icon:hover { border-color: #3b82f6; color: #2563eb; background: #eff6ff; }
.play-btn { color: #10b981; }
.play-btn:hover { border-color: #10b981; color: #059669; background: #ecfdf5; }

.list-toggle { margin-top: 10px; text-align: center; padding-top: 10px; border-top: 1px dashed #e2e8f0; }
.toggle-btn { background: transparent; border: none; color: #64748b; font-size: 0.85rem; font-weight: 600; cursor: pointer; transition: color 0.2s; padding: 5px 15px; border-radius: 20px;}
.toggle-btn:hover { color: #2563eb; background: #f1f5f9; }

.pop-in-enter-active { animation: popIn 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275); }
@keyframes popIn { 0% { transform: scale(0.8); opacity: 0; } 100% { transform: scale(1); opacity: 1; } }
/* 🌟 修复 1：文件选择按钮与徽章美化 */
.outline-dash { background: transparent; border: 2px dashed #93c5fd; color: #2563eb; width: 100%; padding: 1rem; transition: all 0.2s;}
.outline-dash:hover { background: #eff6ff; border-color: #3b82f6; }
.active-file-badge { display: flex; align-items: center; background: #f8fafc; padding: 0.5rem 0.75rem; border-radius: 8px; gap: 0.75rem; border: 1px solid #cbd5e1; }
.file-icon { font-size: 1.4rem; }
.file-name { font-weight: 600; color: #334155; font-size: 0.95rem; flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; font-family: 'Consolas', monospace;}
.badge-actions { display: flex; gap: 0.25rem; border-left: 1px solid #cbd5e1; padding-left: 0.5rem;}
.dir-input { font-family: 'Consolas', monospace; color: #475569; }

/* 🌟 修复 2：单行截断式绝美预期路径卡片 */
.path-preview-card { margin-top: 0.75rem; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; overflow: hidden; }
.preview-header { font-size: 0.75rem; background: #e2e8f0; color: #475569; padding: 4px 8px; font-weight: bold; }
.preview-path { padding: 8px; font-family: 'Consolas', monospace; font-size: 0.8rem; color: #64748b; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; direction: rtl; text-align: left; }
.highlight-dir { color: #2563eb; font-weight: bold; }

/* 🌟 修复 3：复合型单位输入框 */
.duration-input-wrapper { display: flex; align-items: center; gap: 0.75rem; }
.input-with-addon { display: flex; border: 1px solid #d1d5db; border-radius: 6px; overflow: hidden; width: 160px; transition: border-color 0.2s;}
.input-with-addon:focus-within { border-color: #3b82f6; box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.1); }
.input-with-addon .number-input { border: none; padding: 0.5rem; flex: 1; min-width: 0; outline: none; font-weight: bold; color: #1e293b;}
.addon-btn { background: #f1f5f9; border: none; border-left: 1px solid #d1d5db; padding: 0 10px; color: #475569; font-weight: bold; cursor: pointer; transition: background 0.2s; white-space: nowrap; font-size: 0.85rem;}
.addon-btn:hover { background: #e2e8f0; color: #2563eb; }

.duration-hint { font-size: 0.85rem; color: #64748b; }
.duration-badge { margin-top: 0.5rem; font-size: 0.85rem; color: #059669; background: #d1fae5; padding: 0.4rem 0.8rem; border-radius: 6px; display: inline-flex; align-items: center; gap: 0.5rem; width: fit-content;}
.time-val { font-weight: bold; font-family: monospace; font-size: 1rem;}
.text-link { background: none; border: none; color: #2563eb; cursor: pointer; text-decoration: underline; padding: 0;}
.divider { border: 0; border-top: 1px dashed #e2e8f0; margin: 1.5rem 0; }

/* 右侧结果卡片 (保持原样) */
.result-card { background: #f9fafb; }
.empty-state { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #94a3b8; }
.empty-icon { font-size: 3rem; margin-bottom: 0.5rem; opacity: 0.5;}
.result-content { display: flex; flex-direction: column; height: 100%; padding: 1.5rem; }
.result-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 1rem; border-bottom: 1px solid #e5e7eb; padding-bottom: 0.75rem;}
.status-title { font-weight: 700; color: #166534; font-size: 1.1rem;}
.has-error .status-title { color: #dc2626; }
.log-list { flex: 1; overflow-y: auto; list-style: none; padding: 0; margin: 0; font-family: 'Consolas', monospace; font-size: 0.85rem; color: #334155; }
.log-list li { padding: 0.5rem 0; border-bottom: 1px dashed #e2e8f0; }
.check-icon { color: #10b981; font-weight: bold; margin-right: 6px;}
.error-msg { color: #dc2626; font-family: monospace; white-space: pre-wrap; }

@media (max-width: 950px) {
  .workspace-grid { grid-template-columns: 1fr; min-height: auto; overflow-y: visible; }
  .form-card, .result-card { min-height: auto; flex-shrink: 0; }
}

/* 🌟 补充：时长小徽章样式 */
.log-duration {
  font-family: 'Consolas', monospace;
  font-size: 0.75rem;
  color: #64748b;
  background: #f1f5f9;
  padding: 2px 6px;
  border-radius: 4px;
  margin-left: 6px;
  font-weight: 500;
  border: 1px solid #e2e8f0;
}

/* 🌟 补充：带文字的操作按钮基础样式 */
.action-btn {
  display: flex; align-items: center; gap: 4px;
  background: white; border: 1px solid #e2e8f0; color: #64748b;
  font-size: 0.8rem; font-weight: 600; padding: 4px 10px;
  border-radius: 6px; cursor: pointer; transition: all 0.2s;
}

/* ✂️ 继续分割按钮：紫蓝色调 (代表处理与重构) */
.split-btn:hover {
  border-color: #8b5cf6;
  color: #7c3aed;
  background: #f5f3ff;
}

/* 🏷️ 打轴标记按钮：橙黄色调 (代表高亮与标记) */
.marker-btn:hover {
  border-color: #f59e0b;
  color: #d97706;
  background: #fffbeb;
}
</style>