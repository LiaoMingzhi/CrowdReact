import api from '@/api'
import { UserInfoResponse } from '@/api/index'

interface UserState {
  userInfo: UserInfoResponse
}

export default {
  namespaced: true,
  state: (): UserState => ({
    userInfo: {
      email: '',
      firstName: '',
      lastName: '',
      balance: 0,
      registerTime: ''
    }
  }),
  mutations: {
    setUserInfo(state: UserState, userInfo: UserInfoResponse) {
      state.userInfo = userInfo
    },
    updateBalance(state: UserState, balance: number) {
      state.userInfo.balance = balance
    }
  },
  actions: {
    async updateUserInfo({ commit }: { commit: any }) {
      try {
        const response = await api.getUserInfo()
        commit('setUserInfo', response.data.data)
      } catch (error) {
        console.error('Failed to update user info:', error)
      }
    }
  }
} 