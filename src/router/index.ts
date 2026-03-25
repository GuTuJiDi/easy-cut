// src/router/index.ts
import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';
import { appModules } from '../config/modules';
import { useWorkspaceStore } from '../stores/workspace';
// 1. 动态生成功能路由
const dynamicRoutes: RouteRecordRaw[] = appModules.map(mod => ({
    path: mod.path,
    name: mod.id,
    component: mod.component,
}));

// 2. 组合最终路由 (默认重定向到注册表的第一个有效功能)
const routes: RouteRecordRaw[] = [
    {
        path: '/',
        redirect: appModules.find(m => !m.disabled)?.path || '/split'
    },
    ...dynamicRoutes
];

export const router = createRouter({
    history: createWebHashHistory(),
    routes,
});

// 全局路由守卫 (商业化拦截预留处)
router.beforeEach((to, from, next) => {
    const workspaceStore = useWorkspaceStore();
    const targetModule = appModules.find(m => m.id === to.name);

    // 如果目标模块存在，且是 Pro 功能，且用户不是 Pro 版本
    if (targetModule && targetModule.isPro && !workspaceStore.isProVersion) {
        // 这里可以触发一个全局的事件或 Pinia 状态，弹出一个绝美的“升级 PRO”引导弹窗
        alert("🔒 该功能为 PRO 专业版独占，请先升级！");
        next(false); // 拦截跳转
    } else {
        next(); // 正常放行
    }
});