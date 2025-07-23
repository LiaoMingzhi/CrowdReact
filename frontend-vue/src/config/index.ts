interface Config {
  // API配置
  API: {
    BASE_URL: string
    TIMEOUT: number
    RETRY_TIMES: number
  }
  
  // 认证相关
  AUTH: {
    TOKEN_KEY: string
    USERNAME_KEY: string
    TOKEN_EXPIRE_DAYS: number
    TOKEN_EXPIRY_KEY: string
  }
  
  // 钱包相关
  WALLET: {
    ADDRESS_DISPLAY: {
      prefix: number
      suffix: number
    }
  }
  
  // 投注相关
  BETTING: {
    MIN_AMOUNT: number
    MAX_AMOUNT: number
    DEFAULT_AMOUNT: number
  }

  // UI相关
  UI: {
    THEME_COLOR: string
    TABLE_PAGE_SIZES: number[]
    DEFAULT_PAGE_SIZE: number
    MESSAGE_DURATION: number
  }
}

const config: Config = {
  API: {
    BASE_URL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:9080',
    TIMEOUT: 10000, // 请求超时时间（毫秒）
    RETRY_TIMES: 3  // 请求失败重试次数
  },

  AUTH: {
    TOKEN_KEY: 'token',
    USERNAME_KEY: 'username',
    TOKEN_EXPIRE_DAYS: 7,
    TOKEN_EXPIRY_KEY: 'tokenExpiry'
  },

  WALLET: {
    ADDRESS_DISPLAY: {
      prefix: 6,
      suffix: 4
    }
  },

  BETTING: {
    MIN_AMOUNT: 0.01,
    MAX_AMOUNT: 100,
    DEFAULT_AMOUNT: 1
  },

  UI: {
    THEME_COLOR: '#409EFF',
    TABLE_PAGE_SIZES: [10, 20, 50, 100],
    DEFAULT_PAGE_SIZE: 20,
    MESSAGE_DURATION: 3000 // 消息提示持续时间（毫秒）
  }
}

export default config 