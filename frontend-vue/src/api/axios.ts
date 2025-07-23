import axios, { AxiosError, AxiosInstance, InternalAxiosRequestConfig } from 'axios';
import { ElMessage } from 'element-plus';

// 声明模块扩展AxiosRequestConfig类型
declare module 'axios' {
    interface InternalAxiosRequestConfig {
        retryCount?: number;
    }
}

// 创建axios实例
const api: AxiosInstance = axios.create({
    baseURL: import.meta.env.VITE_API_BASE_URL,
    timeout: 15000,
    headers: {
        'Content-Type': 'application/json',
    }
});

// 请求重试配置
const retryDelay = 1000; // 重试延迟(ms)
const maxRetryAttempts = 3; // 最大重试次数

// 添加响应数据类型接口
interface ApiErrorResponse {
    message?: string;
    [key: string]: any;
}

// 请求拦截器
api.interceptors.request.use(
    (config: InternalAxiosRequestConfig) => {
        config.retryCount = 0;
        return config;
    },
    error => Promise.reject(error)
);

// 响应拦截器
api.interceptors.response.use(
    response => response,
    async (error: AxiosError<ApiErrorResponse>) => {
        const config = error.config as any;
        
        // 如果是频率限制错误(429)且未超过最大重试次数
        if (error.response?.status === 429 && config.retryCount < maxRetryAttempts) {
            config.retryCount += 1;
            
            // 使用指数退避算法计算延迟时间
            const delay = retryDelay * Math.pow(2, config.retryCount - 1);
            
            // 添加随机抖动避免同时重试
            const jitter = Math.random() * 1000;
            
            await new Promise(resolve => setTimeout(resolve, delay + jitter));
            
            return api(config);
        }

        // 处理其他错误
        if (error.response?.status === 429) {
            ElMessage.error('请求过于频繁，请稍后再试');
        } else {
            ElMessage.error(error.response?.data?.message || '请求失败');
        }

        return Promise.reject(error);
    }
);

export default api; 