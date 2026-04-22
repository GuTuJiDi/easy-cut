<script setup lang="ts">

</script>

<template>
  <div class="ai-marker-layout split-layout">

    <aside class="left-panel custom-scrollbar">

      <div class="panel-card engine-card">
        <h3 class="panel-title">🧠 核心推理引擎</h3>

        <div v-if="!isModelReady" class="model-download-box">
          <div class="status-icon">📦</div>
          <h4>需要下载 Whisper 模型</h4>
          <p>首次使用需下载基础语义识别模型 (约 140MB)</p>

          <div v-if="isDownloading" class="progress-wrapper">
            <div class="progress-bar-bg">
              <div class="progress-bar-fill" :style="{ width: downloadProgress + '%' }"></div>
            </div>
            <span class="progress-text">{{ downloadProgress }}% ({{ downloadSpeed }} MB/s)</span>
          </div>

          <button v-else class="elegant-btn download-btn" @click="startDownloadModel">
            ⬇️ 立即下载模型
          </button>
        </div>

        <div v-else class="engine-ready-box">
          <div class="engine-status">
            <span class="status-dot is-success"></span>
            <span class="engine-name">Whisper Base (140M)</span>
            <span class="engine-tag">本地离线引擎</span>
          </div>
          <button class="text-btn" @click="switchModel">切换更高精度模型</button>
        </div>
      </div>

      <transition name="fade">
        <div class="panel-card config-card" v-if="isModelReady && videoPath">
          <h3 class="panel-title">⚙️ 识别策略配置</h3>

          <div class="form-group">
            <label>🗣️ 视频主语种</label>
            <select v-model="aiConfig.language" class="pro-select" :disabled="isInferencing">
              <option value="auto">🤖 自动探测语言</option>
              <option value="zh">🇨🇳 中文 (简体/繁体)</option>
              <option value="en">🇺🇸 英文</option>
            </select>
          </div>

          <div class="form-group">
            <label>📏 切片断句策略</label>
            <select v-model="aiConfig.max_length" class="pro-select" :disabled="isInferencing">
              <option value="short">短平快 (适合快节奏解说/短视频)</option>
              <option value="long">自然语义 (适合访谈/长纪录片)</option>
            </select>
          </div>

          <button class="elegant-btn extract-audio start-ai-btn"
                  :disabled="isInferencing"
                  @click="startAIInference">
            <span v-if="isInferencing" class="spinner-small white"></span>
            <span v-else class="btn-icon">✨</span>
            {{ isInferencing ? 'AI 脑力全开中...' : '开始智能语音打轴' }}
          </button>
        </div>
      </transition>
    </aside>

    <main class="right-panel">

      <div v-if="!isModelReady || !videoPath" class="empty-placeholder">
        <div class="placeholder-icon">🤖</div>
        <p>配置好左侧的 AI 引擎与视频后，见证魔法时刻</p>
      </div>

      <div class="right-content-wrapper" v-else>

        <div class="inference-dashboard" v-if="isInferencing || aiLogs.length > 0">
          <h3 class="panel-title">
            <span class="pulse-icon">⚡</span> 实时推理流水线
            <span class="timer">⏱️ 耗时: {{ inferenceTime }}s</span>
          </h3>

          <div class="pipeline-steps">
            <div class="step" :class="{'active': currentStage === 'extract', 'done': currentStage > 'extract'}">
              1. 降采样音频提取 (16kHz)
            </div>
            <div class="step" :class="{'active': currentStage === 'infer', 'done': currentStage > 'infer'}">
              2. Whisper 语义张量解码
            </div>
            <div class="step" :class="{'active': currentStage === 'map', 'done': currentStage > 'map'}">
              3. 序列化注入时间轴
            </div>
          </div>

          <div class="terminal-container">
            <div class="terminal-body custom-scrollbar">
              <pre v-for="(log, idx) in aiLogs" :key="idx">{{ log }}</pre>
            </div>
          </div>
        </div>

        <transition name="fade">
          <div class="ai-results-panel" v-if="!isInferencing && generatedMarkers.length > 0">
            <div class="results-header">
              <h3>🎉 识别完成，共生成 {{ generatedMarkers.length }} 个高亮片段</h3>
              <button class="elegant-btn mini-btn" @click="mergeToManualMarkers">
                📥 一键导入手工打轴列表
              </button>
            </div>

            <div class="marker-list custom-scrollbar">
              <div class="marker-item" v-for="(m, i) in generatedMarkers" :key="i">
                <span class="time-tag">{{ formatTime(m.startTime) }} - {{ formatTime(m.endTime) }}</span>
                <span class="text-content">{{ m.label }}</span>
                <span class="ai-badge">AI</span>
              </div>
            </div>
          </div>
        </transition>

      </div>
    </main>
  </div>
</template>

<style scoped>

</style>