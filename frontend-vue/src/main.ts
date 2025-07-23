import { createApp } from 'vue'
import App from './App.vue'
import router from './router'
import store from './store'
import ElementPlus from 'element-plus'
import 'element-plus/dist/index.css'
import { createPinia } from 'pinia'


const app = createApp(App)
const pinia = createPinia()

app.use(router)
app.use(pinia)
app.use(store)
app.use(ElementPlus)

app.mount('#app') 