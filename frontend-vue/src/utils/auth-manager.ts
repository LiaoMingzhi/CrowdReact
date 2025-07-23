import { useUserStore } from '@/stores/user'

export class AuthManager {
  private static readonly TOKEN_KEY = 'token'
  private static readonly WALLET_KEY = 'walletAddress'
  private static readonly TOKEN_EXPIRY_KEY = 'tokenExpiry'
  
  // 设置 token 和过期时间（7天）
  static setToken(token: string): void {
    localStorage.setItem(this.TOKEN_KEY, token)
    const expiry = Date.now() + 7 * 24 * 60 * 60 * 1000 // 7 days
    localStorage.setItem(this.TOKEN_EXPIRY_KEY, expiry.toString())
  }

  // 检查 token 是否过期
  static isTokenExpired(): boolean {
    const expiry = localStorage.getItem(this.TOKEN_EXPIRY_KEY)
    if (!expiry) return true
    return Date.now() > parseInt(expiry)
  }

  // 清除登录状态
  static clearAuth(): void {
    localStorage.removeItem(this.TOKEN_KEY)
    localStorage.removeItem(this.WALLET_KEY)
    localStorage.removeItem(this.TOKEN_EXPIRY_KEY)
  }
}