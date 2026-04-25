<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="visible" class="modal-mask" @click.self="$emit('close')">
        <div class="modal-container">
          <div class="modal-header">
            <h3>{{ title }}</h3>
            <button @click="$emit('close')" class="close-btn">&times;</button>
          </div>

          <div class="modal-content">
            <slot>
              <p>{{ code }}</p>
            </slot>
          </div>

          <div class="modal-footer">
            <button @click="$emit('close')" class="btn-confirm">我知道了</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup>
import {invoke} from "@tauri-apps/api/core";
import {onMounted, ref} from "vue";

defineProps({
  visible: Boolean, // 控制显示隐藏
  title: {
    type: String,
    default: '详情内容'
  }
});
let code = ref('');
async function testGetMechineCode(){
  try {
    code = await invoke('get_machine_code');
    console.log(code);
  }catch(err){
    console.log(err);
    alert(err)
  }
}
onMounted(() => {
  testGetMechineCode();
})
defineEmits(['close']); // 定义关闭事件
</script>

<style scoped>
.modal-mask {
  position: fixed;
  z-index: 9999;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  justify-content: center;
  align-items: center;
}

.modal-container {
  width: 90%;
  max-width: 550px;
  max-height: 70vh;
  background: #fff;
  border-radius: 12px;
  overflow: hidden; /* 确保圆角不被内容遮挡 */
  display: flex;
  flex-direction: column;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
}

.modal-header {
  padding: 1rem 1.5rem;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #f0f0f0;
}

.modal-content {
  padding: 1.5rem;
  overflow-y: auto; /* 核心：开启长文本滚动 */
  line-height: 1.8;
  font-size: 15px;
  color: #444;
  white-space: pre-wrap; /* 保留换行符 */
}

.modal-footer {
  padding: 1rem 1.5rem;
  text-align: right;
  border-top: 1px solid #f0f0f0;
}

.btn-confirm {
  background: #409eff;
  color: white;
  border: none;
  padding: 8px 20px;
  border-radius: 6px;
  cursor: pointer;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: #999;
  cursor: pointer;
}

/* 动画 */
.fade-enter-active, .fade-leave-active { transition: opacity 0.25s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>