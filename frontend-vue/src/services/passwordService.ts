import { generatePasswordFromAddress } from "@/utils/auth"

export const passwordService = {
  // 生成密码
  generatePassword(address: string): string {
    return generatePasswordFromAddress(address)
  },

  // 保存密码到本地存储
  savePassword(username: string, password: string): void {
    localStorage.setItem(`pwd_${username.toLowerCase()}`, password)
  },

  // 获取保存的密码
  getPassword(username: string): string | null {
    return localStorage.getItem(`pwd_${username.toLowerCase()}`)
  },

  // 清除密码
  clearPassword(username: string): void {
    localStorage.removeItem(`pwd_${username.toLowerCase()}`)
  }
} 