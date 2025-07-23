import { ContractService } from '@/api/contract'
import { ref } from 'vue'

const contractService = ref<ContractService | null>(null)

export function useContractService() {
  if (!contractService.value) {
    contractService.value = new ContractService()
  }
  return contractService.value
} 