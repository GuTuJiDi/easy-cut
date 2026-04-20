// src/router/index.ts
import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router';
import { appModules } from '../config/modules';
import { useAuthStore } from '../stores/auth'; // 🌟 1. 引入真正的鉴权 Store
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

// 全局路由守卫 (商业化拦截真实生效处)
router.beforeEach((to, from, next) => {
    // 🌟 2. 实例化真正的鉴权 Store
    const authStore = useAuthStore();
    const targetModule = appModules.find(m => m.id === to.name);

    // 🌟 3. 核心修复：使用 authStore.isPro 来判断
    if (targetModule && targetModule.isPro && !authStore.isPro) {
        // 如果目标是 PRO 功能且用户未激活，弹窗并拦截跳转
        alert("🔒 该功能为 PRO 专业版独占，请先前往设置中心激活！");
        next(false); // 拒绝跳转
    } else {
        next(); // 放行
    }
});