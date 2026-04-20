import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export const useAuthStore = defineStore('auth', () => {
    const isPro = ref(false);
    const licenseKey = ref('');

    async function verifyAndActivate(key: string): Promise<void> {
        try {
            const isValid = await invoke<boolean>('verify_license_cmd', { licenseStr: key });
            if (isValid) {
                isPro.value = true;
                licenseKey.value = key;
            } else {
                throw new Error("签名无效或已篡改");
            }
        } catch (error) {
            deactivate();
            throw error;
        }
    }

    // 🌟 新增：静默查票机制 (防篡改、防过期)
    async function silentVerify() {
        if (!licenseKey.value) {
            isPro.value = false;
            return;
        }
        try {
            // 拿着本地的钥匙，去 Rust 底层重新验一次票
            const isValid = await invoke<boolean>('verify_license_cmd', { licenseStr: licenseKey.value });
            if (!isValid) {
                console.warn("授权已过期或本地状态被篡改，已强制降级为免费版。");
                deactivate();
            } else {
                isPro.value = true; // 确保状态正确
            }
        } catch (e) {
            deactivate();
        }
    }

    function deactivate() {
        isPro.value = false;
        licenseKey.value = '';
    }

    return { isPro, licenseKey, verifyAndActivate, silentVerify, deactivate };
}, {
    persist: true // 魔法：一键开启持久化
});