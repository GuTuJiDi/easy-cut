import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
// 🌟 引入你的 HTTP 请求客户端 (以 axios 示意)
// import api from '@/utils/api';
// 🌟 核心重构：废除 persist: true，不再将鉴权状态存入 localStorage
export const useAuthStore = defineStore('auth', () => {
    const isPro = ref(false);
    const sessionToken = ref(''); // 🛡️ 本次会话的唯一令牌
    const isInitialized = ref(false); // 保持初始化锁，防止路由竞态

    // 🌟 优化：将悬浮球模式判断统一收拢到 Store
    const isWidget = ref(window.location.hash.includes('widget'));

// 假设你有一个获取当前设备指纹/ID的方法或状态
//     const deviceId = ref('YOUR_DEVICE_UNIQUE_ID');
    // 🌟 核心：应用启动时，一键完成“令牌获取” + “静默查票”
    async function initSecureSession() {
        try {
            // 后端会在返回 Token 的同时，自动查 Keyring 并更新 Rust 内存状态
            sessionToken.value = await invoke('init_security_session');

            // 为了让 Vue 的 UI 知道结果，我们需要额外问一句后端的最终状态
            isPro.value = await invoke('get_pro_status_safe', { sessionToken: sessionToken.value });
        } catch (e) {
            console.error("安全会话初始化失败", e);
            isPro.value = false;
        } finally {
            isInitialized.value = true;
        }
    }

    async function activatePro(key: string): Promise<void> {
        try {
            // 用户输入激活码，后端验证并写入操作系统 Keyring
            const success = await invoke<boolean>('verify_license_and_activate', {
                licenseStr: key
            });
            isPro.value = success;
        } catch (error) {
            isPro.value = false;
            throw error;
        }
    }
    async function deactivatePro() {
        try {
            // 调用后端指令物理清理
            await invoke('deactivate_license_safe', {
                sessionToken: sessionToken.value
            });
            isPro.value = false;
        } catch (e) {
            throw e;
        }
    }
// 🌟 核心重构：严格贯彻 "先云端，后本地"
    /*async function refactor_deactivatePro() {
        try {
            // 步骤 1：先向云端发送解绑请求，释放数据库中的设备配额
            const cloudResponse = await api.post('/v1/license/unbind', {
                device_id: deviceId.value,
                session_token: sessionToken.value
            });

            // 依据后端返回的状态码判断是否真正解绑成功
            if (cloudResponse.data.code !== 200) {
                throw new Error(cloudResponse.data.message || "云端解绑失败，请检查网络");
            }

            // 步骤 2：云端明确返回成功后，才调用 Tauri 后端指令进行本地物理清理
            await invoke('deactivate_license_safe', {
                sessionToken: sessionToken.value
            });

            // 步骤 3：物理层清理成功后，安全翻转内存状态
            isPro.value = false;
        } catch (e) {
            // 任何异常（断网、云端拒绝、本地I/O报错）都会抛出并中断执行。
            // isPro 保持为 true，防止本地变免费、云端坑位仍被占用的死锁。
            throw e;
        }
    }*/
    return { isPro, sessionToken, isInitialized, initSecureSession, activatePro ,deactivatePro,isWidget};
}); // 💡 删除了 persist 配置