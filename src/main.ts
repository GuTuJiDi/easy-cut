import { createApp } from 'vue';
import { createPinia } from 'pinia';
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'; // 🌟 引入插件
import { router } from './router';
import App from './App.vue';
import './assets/global.css';
import devtoolsDetector from 'devtools-detector'; // 🌟 引入探针
const app = createApp(App);
const pinia = createPinia();

// 🌟 将持久化插件挂载到全局 Pinia 实例上
pinia.use(piniaPluginPersistedstate);

app.use(pinia);
app.use(router);
// ==========================================
// 🛡️ 铁穹架构：前端反调试与自我保护机制
// ==========================================
if (import.meta.env.PROD) { // 仅在生产打包环境下生效

    // 1. 禁用所有键盘上的调试快捷键 (F12, Ctrl+Shift+I 等)
    window.addEventListener('keydown', (e) => {
        if (
            e.key === 'F12' ||
            (e.ctrlKey && e.shiftKey && (e.key === 'I' || e.key === 'J' || e.key === 'C')) ||
            (e.ctrlKey && e.key === 'U')
        ) {
            e.preventDefault();
            return false;
        }
    });

    // 2. 禁用右键菜单 (防止右键 -> 检查)
    window.addEventListener('contextmenu', (e) => {
        e.preventDefault();
        return false;
    });

    // 3. 部署高级 DevTools 探针
    devtoolsDetector.addListener((isOpen) => {
        if (isOpen) {
            // 🚨 检测到调试器开启！执行焦土策略！
            console.error("检测到非法调试行为，系统已熔断！");

            // 策略 A：直接清空整个页面 (白屏)
            document.body.innerHTML = '<div style="color:red; text-align:center; padding-top:20%; font-size:24px;">🚫 检测到非法调试环境，为保护数据安全，应用已终止。请重启软件。</div>';

            // 策略 B：触发死循环卡死浏览器的渲染进程 (极其恶心)
            setInterval(() => {
                debugger;
            }, 50);

            // 策略 C (可选)：通知 Rust 后端，直接强制关闭 Tauri 窗口
            // import { invoke } from '@tauri-apps/api/core';
            // invoke('force_close_app');
        }
    });

    // 启动探针监听
    devtoolsDetector.launch();
}
app.mount('#app');