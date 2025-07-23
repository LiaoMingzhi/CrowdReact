import { createStore } from 'vuex'
import { apiService, type CommissionDetailResponse } from '@/api/backend'

export interface UserInfo {
  email: string
  firstName: string
  lastName: string
  balance: number
  registerTime: string
}

export interface AgentInfo {
  role: string
  upline: string
  downline: string
  totalCommission: number
  ranking: number
  inviteCode: string
}

export interface RootState {
  isLoggedIn: boolean
  user: {
    userInfo: UserInfo
  }
  agentInfo: AgentInfo
  commissionDetails: CommissionDetailResponse[]
}

export default createStore<RootState>({
  state: {
    isLoggedIn: false,
    user: {
      userInfo: {
        email: '',
        firstName: '',
        lastName: '',
        balance: 0,
        registerTime: ''
      }
    },
    agentInfo: {
      role: '',
      upline: '',
      downline: '',
      totalCommission: 0,
      ranking: 0,
      inviteCode: ''
    },
    commissionDetails: []
  },
  mutations: {
    setLoginState(state, isLoggedIn: boolean) {
      state.isLoggedIn = isLoggedIn
    },
    setUserInfo(state, userInfo: UserInfo) {
      state.user.userInfo = userInfo
    },
    setAgentInfo(state, agentInfo: AgentInfo) {
      state.agentInfo = agentInfo
    },
    setCommissionDetails(state, details: CommissionDetailResponse[]) {
      state.commissionDetails = details
    }
  },
  actions: {
    async login({ commit }, credentials) {
      try {
        const response = await apiService.login(credentials)
        if (response.data.status === 'success') {
          commit('setLoginState', true)
          await this.dispatch('updateUserInfo')
          return response
        }
        throw new Error(response.data.message)
      } catch (error) {
        console.error('Login failed:', error)
        throw error
      }
    },
    async register({ commit }, userData) {
      try {
        const response = await apiService.register(userData)
        if (response.data.status === 'success') {
          return response
        }
        throw new Error(response.data.message)
      } catch (error) {
        console.error('Registration failed:', error)
        throw error
      }
    },
    async updateUserInfo({ commit }) {
      try {
        const response = await apiService.getUserInfo()
        if (response.data.status === 'success') {
          commit('setUserInfo', response.data.data)
        }
      } catch (error) {
        console.error('Failed to update user info:', error)
        throw error
      }
    },
    async fetchAgentInfo({ commit }) {
      try {
        const response = await apiService.getAgentInfo()
        if (response.data.status === 'success') {
          commit('setAgentInfo', response.data.data)
        }
      } catch (error) {
        console.error('Failed to fetch agent info:', error)
      }
    },
    async fetchCommissionDetails({ commit }) {
      try {
        const response = await apiService.getCommissionDetails()
        if (response.data.status === 'success') {
          commit('setCommissionDetails', response.data.data)
        }
      } catch (error) {
        console.error('Error fetching commission details:', error)
        throw error
      }
    }
  }
}) 