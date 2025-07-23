import { defineStore } from 'pinia'
import backend, { apiService, type LoginCredentials, type LoginResponseData, type UserInfo, type UserData, type ChangePasswordData } from '@/api/backend'
import { generatePasswordFromAddress } from '@/utils/auth'
import { sha256 } from '@/utils/crypto'
import { AuthManager } from '@/utils/auth-manager'

export interface RegisterResult {
  status: 'success' | 'exists' | 'error'
  message?: string
}

export const useUserStore = defineStore('user', {
  state: () => ({
    token: localStorage.getItem('token') || '',
    walletAddress: localStorage.getItem('walletAddress') || '',
    userInfo: null as UserInfo | null
  }),

  getters: {
    isLoggedIn: (state) => !!state.token,
  },

  actions: {
    generateConsistentPassword(address: string): string {
      const normalizedAddress = address
      return sha256(normalizedAddress)
    },

    async register(username: string, password: string) {
      return await backend.register({
        username: username,
        email: `${username}@placeholder.com`,
        password: password
      })
    },
    
    async login(credentials: { username: string; password: string }) {
      try {
        const response = await backend.login({
          username: credentials.username,
          password: credentials.password
        })

        if (response.data?.status === 'success' && response.data?.data?.token) {
          const token = response.data.data.token
          this.token = token
          this.walletAddress = credentials.username
          AuthManager.setToken(token)
          localStorage.setItem('walletAddress', credentials.username)
          return response
        }
        throw new Error('Invalid login response')
      } catch (error) {
        console.error('Login error:', error)
        throw error
      }
    },

    async fetchUserInfo() {
      try {
        const response = await apiService.getUserInfo()
        if (response.data.status === 'success' && response.data.data) {
          this.userInfo = response.data.data
        } else {
          throw new Error(response.data.message || 'Failed to fetch user info')
        }
      } catch (error) {
        console.error('Failed to fetch user info:', error)
        if (error instanceof Error) {
          throw new Error(error.message)
        } else {
          throw new Error('Failed to fetch user info')
        }
      }
    },

    logout() {
      this.token = ''
      this.walletAddress = ''
      this.userInfo = null
      AuthManager.clearAuth()
    },

    async changePassword(currentPassword: string, newPassword: string) {
      try {
        console.log('Store: Initiating password change')
        const response = await apiService.changePassword({
          currentPassword,
          newPassword
        })
        console.log('Store: Password change response:', response.data)
        return response.data
      } catch (error) {
        console.error('Store: Password change error:', error)
        throw error
      }
    }
  }
}) 