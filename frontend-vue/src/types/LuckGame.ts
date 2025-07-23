export interface LuckGameContract {
  placeBet(number: number): Promise<any>
  withdraw(): Promise<any>
  withdrawCommission(): Promise<any>
  addAgent(agent: string): Promise<any>
  removeAgent(agent: string): Promise<any>
  setAgentRelation(user: string, agent: string): Promise<any>
  getBalance(user: string): Promise<bigint>
  getAgentCommission(agent: string): Promise<bigint>
  isAgent(user: string): Promise<boolean>
  getAgentForUser(user: string): Promise<string>
} 