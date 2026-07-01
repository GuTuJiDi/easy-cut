import { createApp } from 'vue';
import { createPinia } from 'pinia';
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'; // 🌟 引入插件
import { router } from './router';
import App from './App.vue';
import './assets/global.css';

const app = createApp(App);
const pinia = createPinia();

// 🌟 将持久化插件挂载到全局 Pinia 实例上
pinia.use(piniaPluginPersistedstate);

app.use(pinia);
app.use(router);
app.mount('#app');