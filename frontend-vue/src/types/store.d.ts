import { Store } from 'vuex'

interface UserInfo {
  email: string
  firstName: string
  lastName: string
  balance: number
  registerTime: string
}

interface BetInfo {
  amount: number
  address: string
  number: string
  time: string
}

interface AgentInfo {
  role: string
  upline: string
  downline: string
  totalCommission: number
  ranking: number
  inviteCode: string
}

interface RootState {
  isLoggedIn: boolean
  betInfo: BetInfo
  agentInfo: AgentInfo
  commissionDetails: any[]
  user: {
    userInfo: UserInfo
  }
}

declare module '@vue/runtime-core' {
  interface ComponentCustomProperties {
    $store: Store<RootState>
  }
} 