import { ethers, Contract, BrowserProvider } from 'ethers'
import LuckGameArtifact from '@/contracts/LuckGame.json'

export interface IContractService {
  getContract(): Promise<Contract>
  getSigner(): Promise<ethers.JsonRpcSigner>
}

interface ContractConfig {
  address: string
  abi: any
  provider?: BrowserProvider
}

const defaultConfig: ContractConfig = {
  address: import.meta.env.VITE_CONTRACT_ADDRESS,
  abi: LuckGameArtifact.abi
}

export class ContractService implements IContractService {
  private contract: Contract | null = null
  private provider: BrowserProvider | null = null
  private config: ContractConfig

  constructor(config: Partial<ContractConfig> = {}) {
    console.log('ContractService constructor:', {
      defaultConfig,
      config,
      envAddress: import.meta.env.VITE_CONTRACT_ADDRESS
    });
    
    this.config = { ...defaultConfig, ...config }
    if (!this.config.address) {
      throw new Error('Contract address is not configured');
    }
    this.initProvider()
  }

  private async initProvider() {
    if (typeof window.ethereum !== 'undefined') {
      this.provider = new ethers.BrowserProvider(window.ethereum)
      
      // 验证网络
      const network = await this.provider.getNetwork()
      console.log('Connected to network:', network.toString());
      
      const expectedChainId = import.meta.env.VITE_CHAIN_ID
      if (network.chainId.toString() !== expectedChainId) {
        throw new Error(`Please connect to the correct network. Expected chain ID: ${expectedChainId}`);
      }
    } else {
      throw new Error('MetaMask not installed');
    }
  }

  async getContract() {
    if (!this.contract) {
      if (!this.provider) {
        throw new Error('Provider not initialized');
      }
      if (!this.config.address) {
        throw new Error('Contract address is not configured');
      }
      console.log('Creating contract with:', {
        address: this.config.address,
        abi: this.config.abi
      });
      
      const signer = await this.provider.getSigner()
      this.contract = new Contract(
        this.config.address,
        this.config.abi,
        signer
      )
    }
    return this.contract
  }

  async getSigner() {
    if (!this.provider) {
      throw new Error('Provider not initialized');
    }
    await this.provider.send('eth_requestAccounts', [])
    return this.provider.getSigner()
  }
}

// 导出单例实例
export default new ContractService()