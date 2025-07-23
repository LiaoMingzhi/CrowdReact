import { BettingService } from '@/api/betting'
import { useContractService } from './useContractService'

export function useBettingService() {
  const contractService = useContractService()
  return new BettingService(contractService)
} 