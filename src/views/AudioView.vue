<script setup lang="ts">

</script>

<template>
  <div class="view-container"> <header class="view-header">...</header>

    <div class="card"> 内容放这里，文字用 min-width: 0 防溢出
    </div>
  </div>
</template>

<style scoped>
/* --- 新增：小屏幕响应式保护 (极度缩小窗口时的完美形态) --- */
@media (max-width: 800px) {
  .setting-item {
    flex-direction: column; /* 空间不够时，左右结构变成上下结构 */
    align-items: flex-start;
    gap: 1rem;
  }
  .item-content { padding-right: 0; width: 100%; }
  .item-action { width: 100%; }
  .workspace-action { max-width: 100%; } /* 输入框占满整行 */
}
</style>