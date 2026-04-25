import { message } from '@tauri-apps/plugin-dialog';
/**
 * 🌟 铁穹前端熔断器
 * 拦截到底层抛出的鉴权失败异常时，强制刷新页面重置内核状态机
 */
export async function handleSecurityBreach(error: any): Promise<boolean> {
    const errorStr = String(error);

    // 匹配后端 auth.rs 中抛出的典型拦截词汇
    if (errorStr.includes('非法') || errorStr.includes('过期') || errorStr.includes('拒绝访问') || errorStr.includes('Unauthorized')) {
        // 🌟 物理告警
        await message("安全令牌失效或环境异常，应用将强制重置。", {
            title: "铁穹安全防御系统",
            kind: 'error'
        });
        // 视觉震慑：清空当前界面，显示拉闸警告
        document.body.innerHTML = `
            <div style="display:flex; flex-direction:column; align-items:center; justify-content:center; height:100vh; background:#0f172a; color:#ef4444; font-family:sans-serif;">
                <h1 style="font-size:3rem; margin-bottom:1rem;">🚫 访问拒绝</h1>
                <p style="font-size:1.2rem; color:#94a3b8;">安全会话已失效或遭到篡改，系统正在重置验证通道...</p>
            </div>
        `;

        // 1.5秒后强制物理刷新，迫使重新走 initSecureSession 分发新 Token
        setTimeout(() => {
            window.location.reload();
        }, 1000);

        return true; // 代表是安全类错误
    }

    return false; // 代表是普通的业务报错（如：找不到文件）
}