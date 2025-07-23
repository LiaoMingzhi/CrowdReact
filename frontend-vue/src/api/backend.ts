import axios from 'axios'
import type { AxiosInstance, AxiosResponse } from 'axios'

export interface ApiResponse<T> {
  status: 'success' | 'error'
  data: T
  message?: string
}

export interface LoginResponseData {
  token: string
  user: {
    id: number
    username: string
    email: string
  }
}

export interface LoginCredentials {
  username: string
  password: string
}

export interface UserData {
  username: string
  email: string
  password: string
}

export interface UserInfo {
  email: string
  firstName: string
  lastName: string
  balance: number
  registerTime: string
}

export interface AgentInfo {
  id: string
  userAddress: string
  levelAgent: string
  superiorAgent: string
  ranking: number
  inviteCode: string
}

export interface AgentDetails {
  one_agent: {
    user_address: string
    level_agent: string
    created_at: string
  } | null,
  two_agents: Array<{
    user_address: string
    level_agent: string
    created_at: string
  }>,
  common_agents: Array<{
    user_address: string
    level_agent: string
    created_at: string
  }>
}

export interface AgentInfoResponse {
  user_address: string
  level_agent: string
  agent_details: AgentDetails
}

export interface CommissionDetail {
  user_address: string
  from_address: string
  commission: string
  transaction_hash: string
  time: string
}

export interface CommissionDetailResponse {
  user_address: string
  commission_details: CommissionDetail[]
  total?: number
  total_amount?: string
}

export interface CommissionDetailParams {
  page?: number
  limit?: number
  startDate?: string
  endDate?: string
}

export interface BetRecord {
  id: number
  account_address: string
  amount: string
  bet_number: number
  transaction_hash: string
  block_number: number
  created_at: string
  status: 'win' | 'lose'
}

export interface ChangePasswordData {
  currentPassword: string
  newPassword: string
}

export interface BetHistoryResponse {
  user_address: string
  bet_history: {
    amount: string
    transaction_hash: string
    luck_number: string[]
    time: string
  }[]
}

export interface BetHistoryParams {
  page?: number
  limit?: number
  startDate?: string
  endDate?: string
}

export interface PrizePoolPrizeExpectionResponse {
  prize_amount: string
  first_prize: string
  second_prize: string
  third_prize: string
  level_one_agent_prize: string
  level_two_agent_prize: string
  common_agent_prize: string
}

export interface LotteryInfo {
  user_address: string
  prize_grade: 'first_prize' | 'second_prize' | 'third_prize' | 'level_one_agent' | 'level_two_agent' | 'level_common_agent'
  prize_amount: number
  time: string
}

export interface CompetitionInfo {
  user_address: string
  topic: string
  rank: number
  competition_count: number
}

export interface HomeInfoResponse {
  competition: CompetitionInfo
  lottery: LotteryInfo[]
}

export const api: AxiosInstance = axios.create({
  baseURL: '/api',
  timeout: 5000,
  headers: {
    'Content-Type': 'application/json'
  }
})

// 请求拦截器
api.interceptors.request.use(
  config => {
    const token = localStorage.getItem('token')
    // console.log('Token from localStorage:', token)
    
    if (token) {
      const tokenValue = token.startsWith('Bearer ') ? token : `Bearer ${token}`
      config.headers.Authorization = tokenValue.trim()
      // console.log('Authorization header:', config.headers.Authorization)
    } else {
      console.warn('No token found in localStorage')
    }
    return config
  },
  error => {
    console.error('Request interceptor error:', error)
    return Promise.reject(error)
  }
)

export const apiService = {
  login: (credentials: LoginCredentials): Promise<AxiosResponse<ApiResponse<LoginResponseData>>> => {
    return api.post('/auth/login', credentials)
  },

  register: (userData: UserData): Promise<AxiosResponse<ApiResponse<void>>> => {
    return api.post('/auth/register', userData)
  },

  getUserInfo: (): Promise<AxiosResponse<ApiResponse<UserInfo>>> => {
    return api.get('/user/info')
  },

  getAgentDetails: (): Promise<AxiosResponse<ApiResponse<AgentInfoResponse>>> => {
    const account_address = localStorage.getItem('walletAddress')
    return api.get('/bet/agent_details', {
      params: {
        account_address: account_address
      }
    })
  },

  getCommissionDetails: (params?: CommissionDetailParams): Promise<AxiosResponse<ApiResponse<CommissionDetailResponse[]>>> => {
    const account_address = localStorage.getItem('walletAddress')
    return api.get('/bet/commission', {
      params: {
        page: params?.page || 1,
        limit: params?.limit || 10,
        account_address: account_address
      }
    })
  },

  checkBettingHistory: (): Promise<AxiosResponse<ApiResponse<BetRecord[]>>> => {
    return api.get('/bet/history', {
      headers: {
        'Content-Type': 'application/json'
      }
    })
  },

  changePassword: (data: ChangePasswordData): Promise<AxiosResponse<ApiResponse<void>>> => {
    return api.post('/user/change-password', {
      current_password: data.currentPassword,
      new_password: data.newPassword
    })
  },

  getAgents: (): Promise<AxiosResponse<ApiResponse<AgentInfo[]>>> => {
    return api.get('/bet/agents', {
      headers: {
        'Content-Type': 'application/json'
      }
    })
  },

  getBetHistory: (params?: BetHistoryParams): Promise<AxiosResponse<ApiResponse<BetHistoryResponse>>> => {
    const account_address = localStorage.getItem('walletAddress')
    return api.get('/bet/history', {
        params: {
            page: params?.page || 1,
            limit: params?.limit || 10,
            account_address: account_address
        }
    })
  },

  getPrizePool(): Promise<AxiosResponse<ApiResponse<PrizePoolPrizeExpectionResponse>>> {
    return api.get('/prize/pool')
  },

  getHomeInfo(): Promise<AxiosResponse<ApiResponse<HomeInfoResponse>>> {
    const account_address = localStorage.getItem('walletAddress')
    return api.get('/prize/competition_lottery', {
      params: {
        account_address: account_address
      }
    })
  }

}

export default apiService
