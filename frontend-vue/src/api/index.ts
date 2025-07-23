import axios, { AxiosInstance, AxiosResponse } from 'axios'

// 定义响应类型
export interface ApiResponse<T> {
  data: T
  status: number
  message?: string
}

export interface UserInfoResponse {
  email: string
  firstName: string
  lastName: string
  balance: number
  registerTime: string
}

export interface AgentInfoResponse {
  id: string
  userAddress: string
  levelAgent: string
  superiorAgent: string
  ranking: number
  inviteCode: string
}

export interface CommissionDetailResponse {
  id: string
  amount: number
  date: string
  type: string
}

export interface BetHistoryItem {
  time: string
  amount: number
  number: string
  status: 'win' | 'lose'
}

export interface UplineAgentResponse {
  id: string
  name: string
}

export interface LoginResponse {
  token: string
  username: string
  isFirstLogin: boolean
}

export interface Api {
  getUserInfo(): Promise<AxiosResponse<ApiResponse<UserInfoResponse>>>
  getAgentInfo(): Promise<AxiosResponse<ApiResponse<AgentInfoResponse>>>
  getCommissionDetails(): Promise<AxiosResponse<ApiResponse<CommissionDetailResponse[]>>>
  getRandomUplineAgents(): Promise<AxiosResponse<ApiResponse<UplineAgentResponse[]>>>
}

const api: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 5000
})

const apiInstance: Api = {
  getUserInfo() {
    return api.get<ApiResponse<UserInfoResponse>>('/user/info')
  },

  getAgentInfo() {
    return api.get<ApiResponse<AgentInfoResponse>>('/agent/info')
  },

  getCommissionDetails() {
    return api.get<ApiResponse<CommissionDetailResponse[]>>('/commission/details')
  },

  getRandomUplineAgents() {
    return api.get<ApiResponse<UplineAgentResponse[]>>('/agent/random-uplines')
  }
}

export default apiInstance 