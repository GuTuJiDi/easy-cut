import { defineConfig, type ConfigEnv, type UserConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// 🌟 1. 忽略缺少类型定义文件的报错
// @ts-ignore
import obfuscator from 'rollup-plugin-obfuscator';

// 🌟 2. 集中处理 Node.js 全局变量 process，避免下方重复爆红
// @ts-expect-error process is a nodejs global
const host = process.env.TAURI_DEV_HOST;
// @ts-expect-error process is a nodejs global
const tauriPlatform = process.env.TAURI_ENV_PLATFORM;
// @ts-expect-error process is a nodejs global
const tauriDebug = process.env.TAURI_ENV_DEBUG;

// https://vite.dev/config/
// 🌟 3. 显式声明 ConfigEnv 和 UserConfig 类型，解决 mode 爆红
export default defineConfig(async ({ mode }: ConfigEnv): Promise<UserConfig> => ({
    plugins: [vue()],

    // Vite options tailored for Tauri development
    clearScreen: false,
    server: {
        port: 1420,
        strictPort: true,
        host: host || false,
        hmr: host
            ? {
                protocol: "ws",
                host,
                port: 1421,
            }
            : undefined,
        watch: {
            ignored: ["**/src-tauri/**"],
        },
    },

    // ==========================================
    // 🛡️ 铁穹架构：前端焦土策略 (构建与深度混淆)
    // ==========================================
    build: {
        // 使用上方处理好的变量
        target: tauriPlatform == 'windows' ? 'chrome105' : 'safari13',
        sourcemap: !!tauriDebug,
        minify: !tauriDebug ? 'esbuild' : false,

        rollupOptions: {
            plugins: [
                // 🌟 4. 使用数组展开运算符 `...` 解决 undefined 塞入数组导致的类型报错
                ...(mode === 'production' ? [
                    obfuscator({
                        globalOptions: {
                            compact: true,
                            controlFlowFlattening: true,
                            controlFlowFlatteningThreshold: 0.7,
                            deadCodeInjection: true,
                            deadCodeInjectionThreshold: 0.4,
                            debugProtection: true,
                            debugProtectionInterval: 4000,
                            disableConsoleOutput: true,
                            identifierNamesGenerator: 'hexadecimal',
                            renameGlobals: false,
                            stringArray: true,
                            stringArrayEncoding: ['rc4'],
                            stringArrayThreshold: 0.8,
                            unicodeEscapeSequence: false,
                        }
                    })
                ] : [])
            ]
        }
    }
}));