import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:9080',
        changeOrigin: true
      }
    }
  },
  // 添加日志配置
  logLevel: 'info',  // 'info' | 'warn' | 'error' | 'silent'
}) 