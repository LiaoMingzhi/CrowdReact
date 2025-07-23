import { AxiosResponse } from 'axios'

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
  role: string
  upline: string
  downline: string
  totalCommission: number
  ranking: number
  inviteCode: string
}

export interface CommissionDetailResponse {
  id: string
  amount: number
  date: string
  type: string
}

export interface Api {
  getUserInfo(): Promise<AxiosResponse<ApiResponse<UserInfoResponse>>>
  getAgentInfo(): Promise<AxiosResponse<ApiResponse<AgentInfoResponse>>>
  getCommissionDetails(): Promise<AxiosResponse<ApiResponse<CommissionDetailResponse[]>>>
}

declare const api: Api
export default api 